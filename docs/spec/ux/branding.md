# Branding & White-Labelling

## Overview

The RepoRoller web UI supports deployment-time branding so that organizations can present the
tool under their own identity. An organization deploying RepoRoller can provide their logo,
application name, and primary brand colour without modifying source code.

Branding is configuration, not customization. The layout, component structure, and interaction
patterns are fixed. Only the visual tokens and identity markers are configurable.

---

## What Is Configurable

| Brand element | Config key | Type | Default | Applied to |
|---|---|---|---|---|
| Application name | `brand.app_name` | `string` | `"RepoRoller"` | Page titles, headings, sign-in screen |
| Logo image URL | `brand.logo_url` | `string \| null` | `null` (text wordmark used) | AppShell header, sign-in/callback/access-denied cards |
| Logo alt text | `brand.logo_alt` | `string` | `"[brand.app_name] logo"` | `alt` attribute of logo `<img>` |
| Primary colour | `brand.primary_color` | CSS hex string | `"#0969da"` (GitHub blue) | Buttons, selected states, links, progress indicators |

**Intentionally not configurable:**

- Typography (font family, sizes) — fixed for accessible, predictable layout
- Component structure and layout — white-labelling is not arbitrary theming
- Error messages and instructional copy — must match the spec to stay accurate

---

## How Branding Is Configured

Branding values are provided at deployment time via a configuration file that the SvelteKit
server reads at startup. They are **not** user-configurable at runtime and **not** stored in
the browser.

**Configuration source priority** (highest first):

1. Environment variables: `BRAND_APP_NAME`, `BRAND_LOGO_URL`, `BRAND_LOGO_ALT`, `BRAND_PRIMARY_COLOR`
2. `brand.toml` file in the deployment directory
3. Built-in defaults (as listed above)

**Example `brand.toml`:**

```toml
app_name    = "Acme Dev Tools"
logo_url    = "/static/acme-logo.svg"
logo_alt    = "Acme logo"
primary_color = "#d4451a"
```

---

## How Branding Flows Into the UI

The SvelteKit server loads branding config once at startup and makes it available to all pages
via a root layout `load` function. Components that need branding data receive it as props or
read it from a Svelte context set at the root layout.

The primary colour is injected as a CSS custom property on the root `<html>` element:

```html
<html style="--brand-primary: #d4451a;">
```

All interactive elements (buttons, selected cards, focus rings, progress steps, links) reference
`var(--brand-primary)` rather than a hardcoded colour value. This means the entire colour scheme
adapts to a single config value with no component-level changes.

---

## Logo Rendering Rules

| Scenario | Rendered as |
|---|---|
| `logo_url` is set | `<img src="{logo_url}" alt="{logo_alt}" />` — fixed height, width auto |
| `logo_url` is null | `<span class="wordmark">{app_name}</span>` — styled text wordmark |

The logo container reserves a fixed height (e.g., 32px in AppShell header, 48px on sign-in card)
regardless of which variant is rendered, preventing layout shift between deployments.

---

## Impact on Components

### AppShell (CMP-001)

Gains `appName` and `logoUrl` / `logoAlt` props drawn from branding config:

- Header left area: `[logo or wordmark]  [appName]`
- If `logoUrl` is set: logo image shown; `appName` text is still rendered (visually may be hidden
  on narrow screens but always present for accessibility)

### Sign In screen (SCR-001)

The sign-in card shows the same logo/wordmark as the AppShell, centred above the heading.
The `<h1>` reads "Sign in to [appName]".

### OAuth Callback screen (SCR-002) and Access Denied screen (SCR-003)

Same logo/wordmark as the sign-in card, providing visual continuity through the auth flow.

### Page titles

All page `<title>` elements use the pattern `[screen name] — [appName]`.
When `appName` is "Acme Dev Tools", the sign-in page title becomes "Sign in — Acme Dev Tools".

---

## Accessibility Requirement

When a logo image is used:

- The `alt` attribute must describe the logo meaningfully (e.g., "Acme logo"), not be empty
- If the `appName` text is visually hidden alongside the logo, it must remain present in the DOM
  for screen readers (use `class="sr-only"`, not `display: none` or `aria-hidden`)

---

## UX Assertions

Branding assertions are defined in the central assertions file:

- **UX-ASSERT-026** — Logo falls back to text wordmark when `logo_url` is not configured
- **UX-ASSERT-027** — Custom `primary_color` applies to all interactive elements

See [ux-assertions.md — Branding section](ux-assertions.md#branding).
