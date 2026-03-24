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
| `logoUrl` | `string \| null` | No | Configured logo image URL; if null, `appName` is rendered as a text wordmark |
| `logoAlt` | `string` | No | Alt text for the logo image; defaults to `"[appName] logo"` |
| `userLogin` | `string` | Yes | GitHub username of the authenticated user |
| `userAvatarUrl` | `string \| null` | No | GitHub avatar URL; falls back to initials if null |

### Slots

- `default` — page content below the header

### Events emitted

- `signOut()` — user clicked sign out; caller handles session destruction and navigation

### Header layout

```
[ logo/wordmark ]  [ appName ]          [ @username ▾ ]
```

- Logo/wordmark: left-aligned. If `logoUrl` is set, renders `<img>`; otherwise renders styled
  `appName` text. The `appName` text label is always present in the DOM for screen readers even
  when a logo image is shown (may be visually hidden on narrow screens using `class="sr-only"`).
- `appName` text label: shown next to the logo on wider viewports.
- UserBadge: right-aligned.

### CSS hook

The root `<html>` element has `--brand-primary` set to the configured primary colour.
All interactive elements in this component (and all other components) reference
`var(--brand-primary)` for colour values.

### Accessibility

- Header landmark: `<header>` element
- Main content landmark: `<main>` element wrapping the default slot
- Logo image: `alt` attribute set to `logoAlt`
- When logo image is present, adjacent `appName` text uses `aria-hidden="true"` only if the
  `alt` text already conveys the brand name — otherwise it must remain visible to screen readers
- Sign-out button has explicit label

---

## CMP-001b: BrandCard

**Used by**: SCR-001, SCR-002, SCR-003
**Purpose**: Centred card shell for unauthenticated screens (sign-in, OAuth callback, access denied).
Displays the brand logo/wordmark above the card content, providing visual continuity with the
AppShell before the user is authenticated.

### Props

| Prop | Type | Required | Description |
|---|---|---|---|
| `appName` | `string` | Yes | Configured application name; from branding config |
| `logoUrl` | `string \| null` | No | Configured logo image URL; if null, `appName` rendered as text wordmark |
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
| `name` | `string` | Yes | Template name |
| `description` | `string` | Yes | Template description (clamped to 2 lines by CSS) |
| `tags` | `string[]` | No | Tags to display as chips (max 4 displayed) |
| `repositoryTypeBadge` | `{ typeName: string; policy: 'fixed' \| 'preferable' \| 'optional' } \| null` | No | Repository type badge; null if not applicable |
| `selected` | `boolean` | Yes | Whether this card is the currently selected template |
| `loading` | `boolean` | No | Skeleton/placeholder state while template list loads |

### Events emitted

- `select()` — user clicked this card to select it

### States

- **Default**: card with visible content, not selected
- **Selected**: highlighted border + checkmark icon in top-right corner
- **Hovered**: subtle lift effect (CSS only, no prop needed)
- **Loading**: skeleton placeholders; no events

### Accessibility

- Implemented as a `<label>` wrapping a visually-hidden `<input type="radio">` within the
  template card group; this gives screen readers selection semantics and group context
- `name` of the radio group: `"template-selection"`
- `value` of the radio input: template name
- Card label includes template name and "selected" state for screen readers

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
- Format regex (client-side): `^[a-z0-9][a-z0-9-]*[a-z0-9]$` or single alphanumeric char
  (mirrors GitHub naming rules — lowercase, alphanumeric, hyphens, no leading/trailing hyphens)
- On blur (if format valid): calls `POST /repositories/validate-name`; emits `checking` then result
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

**Used by**: SCR-004 Step 2 (no-variable path) and Step 3; SCR-005
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

### Rendering

- Displayed as an uneditable summary block: "[org] / [name]" with template, type, visibility chips
- "name" field shown as a placeholder if empty (e.g., italicised "—")

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
- Spinner has `role="status"` and `aria-label="Creating repository"`
- When visible, the underlying wizard content has `aria-hidden="true"` to prevent screen
  reader navigation into it
