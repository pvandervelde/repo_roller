# Component Inventory

This document defines the contract for every reusable component in the RepoRoller web UI.
These contracts are inputs for the Interface Designer to translate into typed SvelteKit component
props and events.

Components are listed in dependency order (primitives first, composites last).

---

## CMP-001: AppShell

**Used by**: SCR-004, SCR-005, SCR-006 (authenticated variant)
**Purpose**: Global page layout shell with header containing brand identity and user identity.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `appName` | `string` | Yes | Configured application name (e.g., "Acme Dev Tools"); from branding config |
| `logoUrl` | `string \| null` | No | Light-mode logo image URL; if null, `appName` is rendered as a text wordmark |
| `logoUrlDark` | `string \| null` | No | Dark-mode logo image URL; used in `<picture>` alongside `logoUrl` when set |
| `logoAlt` | `string` | No | Alt text for the logo image; defaults to `"[appName] logo"` |
| `userLogin` | `string` | Yes | GitHub username of the authenticated user |
| `userAvatarUrl` | `string \| null` | No | GitHub avatar URL; falls back to initials if null |

### Slots

- `default` — page content below the header

### Events emitted

- `signOut()` — user clicked sign out; caller handles session destruction and navigation

### Header layout

```
[ logo/wordmark ]  [ appName ]     [ ◑/☀/🌙 ]  [ @username ▾ ]
```

- Logo/wordmark: left-aligned. If `logoUrl` is set, renders `<img>` (or `<picture>` when
  `logoUrlDark` is also set); otherwise renders styled `appName` text. The `appName` text label
  is always present in the DOM for screen readers even when a logo image is shown (may be
  visually hidden on narrow screens using `class="sr-only"`).
- `appName` text label: shown next to the logo on wider viewports.
- Colour scheme toggle: right-aligned, immediately left of the UserBadge.
- UserBadge: right-aligned.

### Colour scheme toggle

The AppShell header contains a colour scheme toggle button placed to the left of the UserBadge
in the header controls area.

**Three-state cycle**: `system → light → dark → system`

| State | Icon | `aria-label` |
|---|---|---|
| `system` | ◑ | "Switch to light mode" |
| `light` | ☀ | "Switch to dark mode" |
| `dark` | 🌙 | "Switch to system colour scheme" |

- **Persistence**: The chosen state is stored in `localStorage` under the key
  `repo-roller-color-scheme`. On page load the stored value is restored; if no value is stored
  the `system` state is used.
- **DOM effect**: The selected state is written as a `data-color-scheme` attribute on the
  `<html>` element (`data-color-scheme="system"`, `"light"`, or `"dark"`). CSS uses this
  attribute alongside `@media (prefers-color-scheme: dark)` to apply the appropriate token
  overrides.

### CSS hook

The root layout injects a `<style>` block that sets `--brand-primary` on `:root` and,
when configured, adds a `@media (prefers-color-scheme: dark)` override. All interactive
elements in this component (and all other components) reference `var(--brand-primary)`
for colour values.

### Accessibility

- Header landmark: `<header>` element
- Main content landmark: `<main>` element wrapping the default slot
- Logo image: `alt` attribute set to `logoAlt`
- When logo image is present, adjacent `appName` text uses `aria-hidden="true"` only if the
  `alt` text already conveys the brand name — otherwise it must remain visible to screen readers
- Sign-out button has explicit label

---

## CMP-001b: BrandCard

> **Naming note**: BrandCard shares the `001` base number with AppShell because it addresses the
> same brand-identity concern — unauthenticated screens cannot use the AppShell header but must
> still display the same logo/wordmark. The `b` suffix makes the structural relationship explicit.
> All other components are numbered sequentially from `002`.

**Used by**: SCR-001, SCR-002, SCR-003
**Purpose**: Centred card shell for unauthenticated screens (sign-in, OAuth callback, access denied).
Displays the brand logo/wordmark above the card content, providing visual continuity with the
AppShell before the user is authenticated.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `appName` | `string` | Yes | Configured application name; from branding config |
| `logoUrl` | `string \| null` | No | Light-mode logo image URL; if null, `appName` rendered as text wordmark |
| `logoUrlDark` | `string \| null` | No | Dark-mode logo image URL; used in `<picture>` alongside `logoUrl` when set |
| `logoAlt` | `string` | No | Alt text for the logo image; defaults to `"[appName] logo"` |

### Slots

- `default` — card body content (heading, message, buttons)

### Layout

```
          [ logo / wordmark ]
   ┌──────────────────────────────┐
   │  (default slot content)      │
   └──────────────────────────────┘
```

Logo/wordmark is centred above the card. Card is centred on the viewport both horizontally and
vertically. No header or footer (intentional — full-screen auth gate).

---

## CMP-002: UserBadge

**Used by**: CMP-001 (AppShell)
**Purpose**: Display the authenticated user's GitHub avatar and username; provides sign-out trigger.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `login` | `string` | Yes | GitHub username |
| `avatarUrl` | `string \| null` | No | GitHub avatar URL |

### Events emitted

- `signOut()` — user clicked "Sign out" in the dropdown

### Accessibility

- Avatar image has `alt="[login]'s GitHub avatar"` or `alt=""` if decorative only (when username is also displayed adjacent to it)
- Dropdown button `aria-haspopup="true"` and `aria-expanded` reflects open state
- Sign-out option is keyboard accessible

---

## CMP-003: StepProgress

**Used by**: SCR-004 (Create Repository wizard)
**Purpose**: Visual step progress indicator for the multi-step wizard.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `steps` | `string[]` | Yes | Step labels in order (e.g., `["Choose template", "Settings", "Variables"]`) |
| `currentStep` | `number` | Yes | 1-indexed current step position |

### Accessibility

- Wrapper has `aria-label="Step [currentStep] of [steps.length]: [currentStepLabel]"`
- Completed steps indicated visually and with `aria-label` on each step dot
- Does not steal focus; focus management is handled by the parent wizard

---

## CMP-004: TemplateCard

**Used by**: CMP-005 (TemplateGrid)
**Purpose**: Selectable card representing a single template.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `name` | `string` | Yes | Template name (raw/slug); used as the radio input's `value` and `aria-label` |
| `displayName` | `string` | Yes | Human-readable display name; rendered in the card's `<h3>` heading (distinct from `name`) |
| `description` | `string` | Yes | Template description (clamped to 2 lines by CSS; expandable — see Show more/Show less below) |
| `tags` | `string[]` | No | Tags to display as chips (max 4 displayed) |
| `repositoryTypeBadge` | `{ typeName: string; policy: 'fixed' \| 'preferable' \| 'optional' } \| null` | No | Repository type badge; null if not applicable |
| `selected` | `boolean` | Yes | Whether this card is the currently selected template |
| `loading` | `boolean` | No | Skeleton/placeholder state while template list loads |

### Events emitted

- `select()` — user clicked this card to select it

### Show more / Show less (description expand/collapse)

- A `<button class="template-card__description-toggle">` beneath the description toggles local
  state (`descriptionExpanded`), independent of `selected`.
- Button label flips between "Show more" and "Show less"; `aria-expanded` reflects the current
  state.
- Expanding adds the `template-card__description--expanded` CSS modifier class, which removes the
  2-line `-webkit-line-clamp` so the full `description` text is shown.
- The toggle's click handler calls `event.stopPropagation()` so clicking it never triggers the
  card's own selection/deselection (the card is a `<label>` wrapping a radio input — without the
  guard, any click inside it would otherwise toggle selection).

### States

- **Default**: card with visible content, not selected
- **Selected**: highlighted border + checkmark icon in top-right corner
- **Hovered**: subtle lift effect (CSS only, no prop needed)
- **Loading**: skeleton placeholders; no events
- **Description collapsed** (default): description clamped to 2 lines; toggle reads "Show more"
- **Description expanded**: full description shown; toggle reads "Show less"; does not affect
  `selected`

### Accessibility

- Implemented as a `<label>` wrapping a visually-hidden `<input type="radio">` within the
  template card group; this gives screen readers selection semantics and group context
- `name` of the radio group: `"template-selection"`
- `value` of the radio input: template name
- Card label includes template name and "selected" state for screen readers
- Description toggle button has `aria-expanded` and is independently focusable/keyboard-operable

### Implementation note

Tag chips and the repository-type badge are rendered internally via the shared `TagList` and
`RepositoryTypeBadge` components (see `docs/catalog.md`) rather than this component's own markup;
this is an internal refactor and does not change CMP-004's props or observable behaviour.

---

## CMP-005: TemplateGrid

**Used by**: SCR-004 Step 1
**Purpose**: Grid of selectable TemplateCards with search filtering and loading/error states.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `templates` | `TemplateSummary[]` | Yes | List of templates from the API |
| `selectedTemplateName` | `string \| null` | No | Name of currently selected template (controlled) |
| `loading` | `boolean` | No | Show skeleton cards |
| `error` | `string \| null` | No | Error message to show instead of cards |

### Events emitted

- `templateSelect(templateName: string)` — user selected a template

### Internal behaviour

- Search bar filters `templates` client-side by matching `name`, `description`, and `tags`
- When search produces no results: shows "No templates match '[query]'" empty state

### Accessibility

- Search bar: `<input type="search">` with label "Search templates"
- Radio group wrapping all cards has `role="radiogroup"` and `aria-label="Available templates"`
- Empty states have `role="status"` (non-urgent announcement)

---

## CMP-006: RepositoryNameField

**Used by**: SCR-004 Step 2
**Purpose**: Text input for repository name with client-side format validation and async
uniqueness checking.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `value` | `string` | Yes | Current input value (controlled) |
| `organization` | `string` | Yes | Org name used in the uniqueness check API call |
| `disabled` | `boolean` | No | Disables the field (e.g., during creation overlay) |

### Events emitted

- `change(value: string)` — fired on every input change
- `validationResult(result: NameValidationResult)` — fired after each completed validation cycle

```typescript
type NameValidationResult =
  | { status: 'idle' }
  | { status: 'invalid_format'; message: string }
  | { status: 'checking' }
  | { status: 'available' }
  | { status: 'taken'; name: string }
  | { status: 'check_failed' };
```

### Internal behaviour

- On type: debounce 300ms → client-side format check
- Format regex (client-side): `^[a-z0-9._-]+$` with additional rules: cannot start with `.`;
  cannot contain `..` sequence (mirrors backend validation — see `api-request-types.md`, Repository
  Names section)
- On blur (if format valid): calls `POST /api/v1/repositories/validate-name`; emits `checking` then result
- On type after a "taken" result: clears taken state immediately (user is correcting the name)

### Accessibility

- `<label>` explicitly associated with `<input>`
- Status messages beneath the field use `role="status"` (checking) or `role="alert"` (errors)
- Field has `aria-describedby` pointing to the status message element

---

## CMP-007: RepositoryTypePicker

**Used by**: SCR-004 Step 2
**Purpose**: Display repository type as either a read-only badge (fixed policy) or an editable
dropdown (preferable/optional policy).

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `policy` | `'fixed' \| 'preferable' \| 'optional'` | Yes | How the template constrains the type choice |
| `templateTypeName` | `string \| null` | No | Type name recommended or required by the template |
| `availableTypes` | `RepositoryTypeOption[]` | Yes | List of types from the API (empty array when loading or unavailable) |
| `selectedTypeName` | `string \| null` | Yes | Currently selected type (controlled) |
| `loading` | `boolean` | No | Show loading state while type list fetches |
| `disabled` | `boolean` | No | Disable interaction |

```typescript
type RepositoryTypeOption = { name: string; description: string };
```

### Events emitted

- `change(typeName: string | null)` — user changed selection

### Rendering rules

- `fixed` policy: renders as a read-only row ("Repository type: [name]" with lock icon); no dropdown
- `preferable` policy: renders as a dropdown pre-selected to `templateTypeName`; user can change
- `optional` policy: renders as a dropdown with "No specific type" as first option

---

## CMP-008: VariableField

**Used by**: SCR-004 Step 3
**Purpose**: Single text input for one template variable.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `variableName` | `string` | Yes | Raw variable name (e.g., `service_name`) |
| `label` | `string` | Yes | Human-readable label derived from `variableName` |
| `description` | `string \| null` | No | Variable description shown as helper text |
| `required` | `boolean` | Yes | Whether the variable must be filled |
| `defaultValue` | `string \| null` | No | Pre-filled if present |
| `value` | `string` | Yes | Current value (controlled) |
| `disabled` | `boolean` | No | Disables the input |

### Events emitted

- `change(value: string)` — fires on every input change

### Accessibility

- `<label>` explicitly associated with `<input>`
- Required fields: `aria-required="true"` and asterisk (*) in label
- Helper text associated via `aria-describedby`

---

## CMP-009: RepositorySummary

**Used by**: SCR-004 Step 2 (no-variable path), Step 3 (Variables, when the template has any),
and the Review step (final step, all paths — this is where the Create action now lives; see
SCR-004 "Step 4: Review and Create").
**Not used by**: SCR-005 — `routes/create/success/+page.svelte` renders its own
success-screen layout directly and does not import this component. (A prior version of this
entry claimed SCR-005 as a consumer; that was incorrect and has been removed.)
**Purpose**: Read-only summary of the repository about to be (or just) created.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `organization` | `string` | Yes | Org name |
| `repositoryName` | `string` | Yes | Repository name (may be empty string while user is typing) |
| `templateName` | `string` | Yes | Selected template name |
| `typeName` | `string \| null` | No | Repository type, if set |
| `visibility` | `'private' \| 'public'` | Yes | Resolved visibility |
| `teamName` | `string \| null` | No | Team name, if set |
| `description` | `string` | No | Template description; shown only on the Review step, where callers pass the selected template's full description |
| `tags` | `string[]` | No | Template tags, rendered as chips via the shared `TagList` component; defaults to `[]` (renders nothing) |
| `variables` | `Array<{ name: string; value: string }>` | No | Every configured template variable and its resolved value (user-entered value, falling back to the variable's default); defaults to `[]`. Previously captured in wizard state but never displayed anywhere — now shown here on the Review step |

### Rendering

- Displayed as an uneditable summary block: "[org] / [name]" with template, type, visibility chips
- "name" field shown as a placeholder if empty (e.g., italicised "—")
- `description`, when provided, is rendered as a plain paragraph beneath the chips
- `tags`, when provided, are rendered as chips via the shared `TagList` component (uncapped — all
  tags shown, unlike CMP-004's 4-tag cap)
- `variables`, when non-empty, are rendered as a `name: value` list beneath the tags, one row per
  configured variable — this is how the Review step satisfies "show every configured variable's
  name and chosen value" (previously nowhere in the wizard)
- All three new props are optional and default to omitted/empty, so existing call sites (Step 2,
  Step 3) that don't pass them render exactly as before

---

## CMP-010: InlineAlert

**Used by**: All screens
**Purpose**: Non-modal status message for errors, warnings, info, or success.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `variant` | `'error' \| 'warning' \| 'info' \| 'success'` | Yes | Visual and semantic variant |
| `message` | `string` | Yes | The alert message text |
| `action` | `{ label: string; onClick: () => void } \| null` | No | Optional inline action link/button |

### Accessibility

- Uses `role="alert"` for `error` and `warning` variants (announced immediately)
- Uses `role="status"` for `info` and `success` variants (polite announcement)
- Action button/link is keyboard accessible

---

## CMP-011: CreationOverlay

**Used by**: SCR-004 (during creation API call)
**Purpose**: Full-area overlay shown while the creation request is in flight.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `visible` | `boolean` | Yes | Whether the overlay is shown |

### Accessibility

- Overlay wrapper has `aria-live="polite"` on the status message
- Spinner has `role="status"` and `aria-label="Creating repository"` (rendered via the shared
  `Spinner` component, CMP-012, with `size="lg"` — see below; CreationOverlay's own markup and
  accessibility behaviour are unchanged)
- When visible, the underlying wizard content has `aria-hidden="true"` to prevent screen
  reader navigation into it

---

## CMP-012: Spinner

**Used by**: CMP-011 (CreationOverlay, `size="lg"`); CMP-013 (TemplateDetailsPanel, `size="sm"`)
**Purpose**: Shared indeterminate loading spinner. Extracted from CreationOverlay's original
private `.creation-overlay__spinner` markup/CSS so any component needing a loading indicator can
reuse it instead of re-implementing the spin animation.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `size` | `'sm' \| 'md' \| 'lg'` | No | Visual size of the spinner; defaults to `"md"` |
| `label` | `string` | No | When provided, the spinner self-announces (see Accessibility); when omitted, the spinner is `aria-hidden="true"` |

### States

- **Labelled** (`label` provided): renders with `role="status"` and `aria-label={label}` — the
  spinner itself is the accessible announcement (e.g. CMP-011's usage: `label="Creating
  repository"`)
- **Unlabelled** (`label` omitted): renders `aria-hidden="true"` — the caller is responsible for
  its own `aria-live`/`role="status"` wrapper and visible text (e.g. CMP-013's loading state,
  which wraps an unlabelled `size="sm"` spinner in its own `role="status" aria-live="polite"` div
  alongside the text "Loading template details…")

### Accessibility

- Exactly one of `role="status"` + `aria-label` (when `label` is set) or `aria-hidden="true"`
  (when it isn't) is applied — never both, to avoid double-announcing the same loading state
- Purely visual animation; conveys no information on its own beyond "something is loading", so a
  nearby text message is still expected wherever this component is used without `label`

---

## CMP-013: TemplateDetailsPanel

**Used by**: SCR-004 Step 1 (sidebar shown beside the template grid)
**Purpose**: Shows full, untruncated details — description, tags, repository-type badge, and
variables — for whichever template is currently selected in Step 1, so the user can inspect a
template fully without leaving the grid or losing their place.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `templateDetails` | `GetTemplateDetailsResponse \| null` | Yes | Full details response for the selected template; `null` when nothing is selected yet |
| `loading` | `boolean` | Yes | True while the details fetch for the currently-selected template is in flight |
| `error` | `string \| null` | No | Error message to show instead of content; defaults to `null` |

### States

- **Empty** (`templateDetails` is `null`, not loading, no error): "Select a template to see its
  details here." (`role="status"`)
- **Loading**: shared `Spinner` (CMP-012, `size="sm"`, unlabelled) + "Loading template details…"
  text, both wrapped in `role="status" aria-live="polite"`
- **Error**: error message, `role="alert"`
- **Content**: heading (`displayName`), full description, optional repository-type badge, full
  tag list, and a list of the template's variables (name, required/optional marker, and —
  whichever are present — description, default value, pattern)

### Rendering

- Heading: `templateDetails.displayName`, in an `<h2>`
- Description: full, untruncated (no 2-line clamp, unlike CMP-004's card)
- Repository-type badge: rendered via the shared `RepositoryTypeBadge` component, only when
  `templateDetails.repository_type` is present; omitted entirely (no placeholder) when absent.
  > As of this release, the backend's `TemplateDetailsResponse` never actually populates
  > `repository_type`, so in production this badge never renders here today — a pre-existing
  > backend gap, not a defect in this component. Out of scope for this task.
- Tags: full, uncapped list via the shared `TagList` component (no `max` passed — contrast with
  CMP-004's 4-tag cap)
- Variables: each rendered with its name, a Required/Optional marker, and (when present) its
  description, default value, and pattern

### Layout

- Rendered as an `<aside aria-label="Template details">`, placed beside `TemplateGrid` (CMP-005)
  in Step 1 only — not used on any other step
- Two-column layout at viewport widths ≥60rem: grid flexes to fill remaining space, panel is a
  fixed 20rem-wide column beside it. Below 60rem: single column, panel stacks below the grid
- Paired with the wizard's Step-1-only width modifier (80rem, vs. 56rem on every other step) — see
  SCR-004's Layout section

### Accessibility

- Landmark: `<aside aria-label="Template details">`
- Loading state: `role="status" aria-live="polite"` wrapping the spinner and text (announces once,
  politely, rather than per-frame)
- Error state: `role="alert"`
- Empty state: `role="status"`
