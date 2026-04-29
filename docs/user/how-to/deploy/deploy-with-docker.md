---
title: "Deploy with Docker Compose"
description: "Run the RepoRoller backend and frontend containers using Docker Compose."
audience: "operator"
type: "how-to"
---

# Deploy with Docker Compose

RepoRoller runs as two containers that must be deployed together on the same Docker network.

## Prerequisites

- Docker Engine and Docker Compose installed
- All four credential values collected:
  - `GITHUB_APP_ID` and `GITHUB_APP_PRIVATE_KEY` (from the GitHub App)
  - `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET` (from the OAuth App)
- Two random secrets generated (run `openssl rand -hex 48` twice):
  - `SESSION_SECRET`
  - `JWT_SECRET`

## docker-compose.yml

```yaml
services:
  backend:
    image: ghcr.io/myorg/repo-roller-api:latest
    networks:
      - internal
    environment:
      GITHUB_APP_ID:            "${GITHUB_APP_ID}"
      GITHUB_APP_PRIVATE_KEY:   "${GITHUB_APP_PRIVATE_KEY}"
      JWT_SECRET:               "${JWT_SECRET}"
      METADATA_REPOSITORY_NAME: ".reporoller"
      API_HOST:                 "0.0.0.0"
      API_PORT:                 "8080"
      RUST_LOG:                 "info"
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider",
             "http://localhost:8080/api/v1/health"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s

  frontend:
    image: ghcr.io/myorg/repo-roller-frontend:latest
    ports:
      - "3000:3000"
    networks:
      - internal
    depends_on:
      backend:
        condition: service_healthy
    environment:
      GITHUB_CLIENT_ID:     "${GITHUB_CLIENT_ID}"
      GITHUB_CLIENT_SECRET: "${GITHUB_CLIENT_SECRET}"
      GITHUB_ORG:           "myorg"
      ORIGIN:               "https://reporoller.myorg.example"
      SESSION_SECRET:       "${SESSION_SECRET}"
      BACKEND_API_URL:      "http://backend:8080"
      NODE_ENV:             "production"

networks:
  internal:
    driver: bridge
```

## Environment variables

### Backend environment variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `GITHUB_APP_ID` | Yes | — | Numeric App ID from the GitHub App settings page |
| `GITHUB_APP_PRIVATE_KEY` | Yes | — | PEM private key as a single-line `\n`-escaped string |
| `JWT_SECRET` | Yes | — | HS256 signing key for backend-issued JWTs. Minimum 32 characters. |
| `METADATA_REPOSITORY_NAME` | No | `.reporoller` | Name of the metadata repository |
| `API_HOST` | No | `0.0.0.0` | Interface to bind |
| `API_PORT` | No | `8080` | Port to listen on |
| `RUST_LOG` | No | `info` | Log level: `error`, `warn`, `info`, `debug`, `trace` |

### Frontend environment variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `GITHUB_CLIENT_ID` | Yes | — | OAuth App client ID |
| `GITHUB_CLIENT_SECRET` | Yes | — | OAuth App client secret |
| `GITHUB_ORG` | Yes | — | GitHub organisation slug (e.g. `myorg`) |
| `ORIGIN` | Yes | — | Public URL of the frontend, e.g. `https://reporoller.myorg.example`. Must match the OAuth App callback URL base. |
| `SESSION_SECRET` | Yes | — | HMAC-SHA256 key for signing session cookies. Minimum 32 characters. |
| `BACKEND_API_URL` | Yes | — | Internal URL of the backend container, e.g. `http://backend:8080` |
| `NODE_ENV` | No | `production` | Set to `production` for container deployments |
| `BRAND_APP_NAME` | No | `RepoRoller` | Application name |
| `BRAND_LOGO_URL` | No | (none) | URL for the light-mode logo |
| `BRAND_LOGO_URL_DARK` | No | (none) | URL for dark-mode logo |
| `BRAND_LOGO_ALT` | No | `<name> logo` | Logo alt text |
| `BRAND_PRIMARY_COLOR` | No | `#0969da` | CSS accent colour |
| `BRAND_PRIMARY_COLOR_DARK` | No | (none) | Dark-mode accent colour |

## Starting and stopping

```bash
# Start in the background
docker compose up -d

# Check status
docker compose ps

# View logs
docker compose logs -f

# Stop
docker compose down
```

## Verifying the deployment

**Backend health check:**

```bash
curl -f http://localhost:8080/api/v1/health
```

**Frontend:**

Open `https://reporoller.myorg.example` in a browser. You should see the Sign In screen.

## Networking

The backend should **not** be exposed directly to the internet. Only the frontend port (`3000`) should be reachable from end-user browsers. Both containers communicate over the private `internal` Docker network.

If deploying behind a reverse proxy (nginx, Caddy, Traefik), proxy `https://reporoller.myorg.example` → `localhost:3000`. Do not expose port 8080 externally.

## Building images from source

```bash
# From the repository root
docker build -t repo-roller-api -f crates/repo_roller_api/Dockerfile .
docker build -t repo-roller-frontend frontend/
```

Update the `image:` fields in `docker-compose.yml` to use your locally built images.

## Related guides

- [Configure branding](configure-branding.md)
- [Environment variables reference](../../reference/environment-variables.md)
