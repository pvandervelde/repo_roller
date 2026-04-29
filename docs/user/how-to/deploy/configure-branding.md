---
title: "Configure branding"
description: "Customise the RepoRoller web UI with your organisation's name, logo, and primary colour."
audience: "operator"
type: "how-to"
---

# Configure branding

The frontend supports custom branding via either environment variables or a `brand.toml` file. Environment variables take precedence over `brand.toml`.

## Option A: Environment variables

Set any of the following environment variables on the frontend container:

| Variable | Default | Description |
|---|---|---|
| `BRAND_APP_NAME` | `RepoRoller` | Application name in the header and page title |
| `BRAND_LOGO_URL` | (none) | URL to the light-mode logo image |
| `BRAND_LOGO_URL_DARK` | (none) | URL to the dark-mode logo (requires `BRAND_LOGO_URL`) |
| `BRAND_LOGO_ALT` | `<app_name> logo` | Alt text for the logo image |
| `BRAND_PRIMARY_COLOR` | `#0969da` | CSS accent colour (hex, rgb, etc.) |
| `BRAND_PRIMARY_COLOR_DARK` | (none) | Accent colour for dark mode |

Example in `docker-compose.yml`:

```yaml
environment:
  BRAND_APP_NAME:       "Acme RepoRoller"
  BRAND_LOGO_URL:       "https://cdn.acme.example/logo.svg"
  BRAND_LOGO_URL_DARK:  "https://cdn.acme.example/logo-dark.svg"
  BRAND_LOGO_ALT:       "Acme logo"
  BRAND_PRIMARY_COLOR:  "#e63946"
```

## Option B: brand.toml file

Copy `frontend/brand.toml.example` from the repository and customise it:

```toml
# brand.toml
app_name          = "Acme RepoRoller"
logo_url          = "https://cdn.acme.example/logo.svg"
logo_url_dark     = "https://cdn.acme.example/logo-dark.svg"
logo_alt          = "Acme logo"
primary_color     = "#e63946"
primary_color_dark = "#ff6b6b"
```

> **Security:** `brand.toml` must **not** be placed inside `frontend/static/`. It is a server-side file and should not be publicly served. Mount it as a volume at `/app/brand.toml`.

Mount the file when running the container:

```bash
docker run -d \
  --name repo-roller-frontend \
  -p 3000:3000 \
  -v /etc/reporoller/brand.toml:/app/brand.toml:ro \
  # ... other env vars ...
  repo-roller-frontend
```

Or in `docker-compose.yml`:

```yaml
frontend:
  volumes:
    - /etc/reporoller/brand.toml:/app/brand.toml:ro
```

## All fields are optional

All branding fields have defaults. If no branding is configured, the application shows the default RepoRoller name and GitHub blue (`#0969da`) as the accent colour with no logo.
