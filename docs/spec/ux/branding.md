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
| Logo image URL (light) | `brand.logo_url` | `string \| null` | `null` (text wordmark used) | AppShell header, sign-in/callback/access-denied cards |
| Logo image URL (dark) | `brand.logo_url_dark` | `string \| null` | `null` (falls back to `logo_url`) | Same as above, used when system dark mode is active |
| Logo alt text | `brand.logo_alt` | `string` | `"[brand.app_name] logo"` | `alt` attribute of logo `<img>` |
| Primary colour (light) | `brand.primary_color` | CSS hex string | `"#0969da"` (GitHub blue) | Buttons, selected states, links, progress indicators |
| Primary colour (dark) | `brand.primary_color_dark` | CSS hex string | `null` (falls back to `primary_color`) | Same targets as `primary_color`, applied when `prefers-color-scheme: dark` |

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

1. Environment variables: `BRAND_APP_NAME`, `BRAND_LOGO_URL`, `BRAND_LOGO_URL_DARK`, `BRAND_LOGO_ALT`, `BRAND_PRIMARY_COLOR`, `BRAND_PRIMARY_COLOR_DARK`
2. `brand.toml` file in the deployment directory
3. Built-in defaults (as listed above)

**Example `brand.toml`:**

```toml
app_name           = "Acme Dev Tools"
logo_url           = "/static/acme-logo-dark-text.svg"   # dark logo mark, for use on light backgrounds
logo_url_dark      = "/static/acme-logo-light-text.svg"  # light logo mark, for use on dark backgrounds
logo_alt           = "Acme logo"
primary_color      = "#d4451a"
primary_color_dark = "#ff8c69"
```

> **Security note**: `brand.toml` is a server-side configuration file. It **must not** be placed
> inside the SvelteKit `static/` (or `public/`) directory — files in that directory are served
> verbatim to all HTTP clients. Place `brand.toml` in the server's working directory outside the
> static file serving root, or supply values exclusively via environment variables.

---

## How Branding Flows Into the UI

The SvelteKit server loads branding config once at startup and makes it available to all pages
via a root layout `load` function. Components that need branding data receive it as props or
read it from a Svelte context set at the root layout.

The primary colour is injected as a CSS custom property via a `<style>` block rendered by the
root layout at server time:

```html
<style>
  :root { --brand-primary: #d4451a; }
  @media (prefers-color-scheme: dark) {
    :root { --brand-primary: #ff8c69; }
  }
</style>
```

When `primary_color_dark` is not configured, the `@media` block is omitted and `--brand-primary`
has a single value for both light and dark mode. All interactive elements (buttons, selected
cards, focus rings, progress steps, links) reference `var(--brand-primary)` rather than a
hardcoded colour value. This means the entire colour scheme adapts to the config with no
component-level changes.

---

## Dark Mode Support

RepoRoller honours the operating-system / browser `prefers-color-scheme` media feature.
Dark mode colours are opt-in at the operator level; the system remains fully usable without
them.

### Colour switching

- If `primary_color_dark` is set, the root layout emits a `@media (prefers-color-scheme: dark)`
  block that overrides `--brand-primary` for dark mode. No JavaScript, no flash, no client-side
  toggle — it is pure CSS.
- If `primary_color_dark` is **not** set, the same `--brand-primary` value is used in both light
  and dark mode. This is acceptable; many brand colours work on both backgrounds.

### Logo switching

- If `logo_url_dark` is set, the component renders a `<picture>` element:

  ```html
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="/static/acme-logo-light-text.svg">
    <img src="/static/acme-logo-dark-text.svg" alt="Acme logo">
  </picture>
  ```

  The `<img>` `src` is the light-mode logo (shown by default and as fallback). The `<source>`
  provides the dark-mode version.

- If `logo_url_dark` is **not** set, a plain `<img>` is used as before (no `<picture>` wrapper).

- If **neither** `logo_url` **nor** `logo_url_dark` is set, the text wordmark is used regardless
  of colour scheme.

### Background and surface colours

Light/dark background and surface colours (card backgrounds, page background, text) are handled
by the component stylesheet via `prefers-color-scheme` CSS media queries. These are **not**
operator-configurable — only the brand accent colour (`--brand-primary`) and logo are.

---

## Logo Rendering Rules

| Scenario | Rendered as |
|---|---|
| Both `logo_url` and `logo_url_dark` set | `<picture>` with dark `<source>` + light `<img>` fallback |
| Only `logo_url` set | `<img src="{logo_url}" alt="{logo_alt}" />` — fixed height, width auto |
| Neither set | `<span class="wordmark">{app_name}</span>` — styled text wordmark |

The logo container reserves a fixed height (32px in AppShell header, 48px on BrandCard)
regardless of which variant is rendered, preventing layout shift between deployments.

> **Note**: `logo_url_dark` without a `logo_url` is not a supported configuration. If only
> `logo_url_dark` is provided it is ignored; the text wordmark is rendered instead.

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
