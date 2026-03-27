# ADR-008: Web UI Deployment-Time Branding

Status: Accepted
Date: 2026-03-24
Owners: RepoRoller team

## Context

RepoRoller is an open-source tool that organizations deploy internally as a self-service
repository creation interface. Organizations want the deployed tool to appear as part of their
own internal tooling ecosystem — using their logo, company name, and brand colours — rather than
presenting as a generic "RepoRoller" product.

The requirement is branding, not arbitrary theming. The layout, component structure, interaction
patterns, and copy are fixed by the UX specification. Only the visual identity markers need to
be configurable.

Constraints:

- No build step per deployment — changing branding must not require recompiling the SvelteKit
  application
- Single mechanism — operators should not need to understand CSS overrides or component internals
- Accessible by default — any brand colour used must meet WCAG contrast requirements on
  white/light backgrounds (text and interactive elements); this is the operator's responsibility
  to verify, but the system must make that verification straightforward
- Consistent across all screens — branding must apply everywhere, including the unauthenticated
  auth screens (sign-in, OAuth callback, access denied)

## Decision

Provide **six deployment-time configuration tokens** consumed at SvelteKit server startup:

| Token | Type | Default |
|---|---|---|
| `app_name` | string | `"RepoRoller"` |
| `logo_url` | string \| null | null |
| `logo_url_dark` | string \| null | null (falls back to `logo_url`) |
| `logo_alt` | string | `"[app_name] logo"` |
| `primary_color` | CSS hex string | `"#0969da"` |
| `primary_color_dark` | CSS hex string | null (falls back to `primary_color`) |

**Configuration source** (priority order, highest first):

1. Environment variables: `BRAND_APP_NAME`, `BRAND_LOGO_URL`, `BRAND_LOGO_URL_DARK`, `BRAND_LOGO_ALT`, `BRAND_PRIMARY_COLOR`, `BRAND_PRIMARY_COLOR_DARK`
2. `brand.toml` file in the deployment working directory
3. Built-in defaults

**Colour propagation**: `primary_color` (and optionally `primary_color_dark`) are written into a
`<style>` block server-rendered by the root layout. When a dark colour is configured, the block
includes a `@media (prefers-color-scheme: dark)` rule that overrides `--brand-primary`:

```html
<style>
  :root { --brand-primary: #d4451a; }
  @media (prefers-color-scheme: dark) {
    :root { --brand-primary: #ff8c69; }
  }
</style>
```

All interactive elements (buttons, selected card borders, step indicators, links, focus rings)
reference `var(--brand-primary)` rather than hardcoded colour values. A single config change
recolours the entire application; two config values cover both colour schemes.

**Logo fallback**: When `logo_url` is null, the `app_name` string is rendered as a styled text
wordmark. When both `logo_url` and `logo_url_dark` are set, a `<picture>` element is used so
the browser selects the appropriate asset based on `prefers-color-scheme`. The layout reserves a
fixed height for the logo area in all cases, preventing layout shift between deployments.

**Dark mode defaults**: Both `logo_url_dark` and `primary_color_dark` are optional. Omitting them
does not break anything — the light variants are used in both modes.

**App name propagation**: `app_name` is made available to all SvelteKit pages via the root
layout `load` function. Page `<title>` elements and `<h1>` headings that include the app name
read from this value, not from a hardcoded string.

## Consequences

**Enables:**

- Organizations deploy RepoRoller under their own visual identity with no code changes
- A single environment variable change at deploy time is sufficient for colour customisation
- Operators can supply dark-mode–specific brand colours and logos without any code changes
- The fallback chain (env vars → file → defaults) supports both containerised and traditional
  deployments
- Adding a seventh token (e.g., `favicon_url`) in future requires no architectural changes

**Forbids:**

- Per-user or per-session branding (configuration is deployment-time only)
- Changing layout, component structure, or interaction patterns through branding config
- Arbitrary CSS injection via the branding config (only the six tokens are supported)
- A user-toggled dark mode switch — mode follows the system `prefers-color-scheme` setting only

**Trade-offs:**

- The operator is responsible for ensuring their chosen `primary_color` meets WCAG AA contrast
  requirements (4.5:1 for normal text, 3:1 for large text and UI components) — the system does
  not validate this
- Server-side rendering means the CSS custom property is always set before first paint; there is
  no flash of default colour

## Alternatives considered

### Option A: CSS override file

Let operators provide a full `custom.css` file mounted into the deployment.

**Why not**: Requires detailed knowledge of the component structure and CSS class names. Brittle
across upgrades. Cannot be expressed as environment variables for container deployments. More
surface area for unintended visual breakage.

### Option B: Build-time configuration (environment variables baked in at `npm run build`)

Bake brand values into the static bundle at build time.

**Why not**: Requires a build pipeline per operator. Builds are not fast. Operators without
Node.js tooling cannot deploy. Defeats the goal of a simple deployment story.

### Option C: Runtime API endpoint for branding

Serve branding values from a `/api/v1/brand` endpoint and apply them client-side after load.

**Why not**: Causes a visible flash of default colours before the brand CSS applies. Adds an
extra round-trip before the page is visually stable. More complex than reading config at
server startup.
