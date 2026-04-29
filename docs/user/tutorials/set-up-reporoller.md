---
title: "Set up RepoRoller for your organization"
description: "First-time deployment walkthrough: create the metadata repo, register the GitHub App and OAuth App, generate secrets, and start containers."
audience: "operator"
type: "tutorial"
---

# Set up RepoRoller for your organization

By the end of this tutorial, RepoRoller will be running and accessible to your organisation. You will create the metadata repository, register two GitHub Apps, generate secrets, write a Docker Compose configuration, and verify a healthy deployment.

**What you need before you start:**

- Owner access to your GitHub organisation
- A server with Docker and Docker Compose installed
- A public-facing hostname for the frontend (e.g. `reporoller.myorg.example`) with TLS termination

Estimated time: 45–60 minutes.

---

## Step 1: Create the metadata repository

The metadata repository stores all configuration that controls how RepoRoller creates repositories.

1. In your GitHub organisation, create a new **private** repository named `.reporoller`.
2. Clone it locally:

```bash
git clone https://github.com/myorg/.reporoller.git
cd .reporoller
```

1. Create the required directory structure:

```bash
mkdir -p global types teams
```

1. Create a minimal global configuration file:

```bash
cat > global/defaults.toml << 'EOF'
[repository]
has_issues  = true
has_projects = false
has_wiki    = false
has_discussions = false

[pull_requests]
allow_merge_commit = false
allow_squash_merge = true
allow_rebase_merge = false
required_approving_review_count = 1
EOF
```

1. Commit and push:

```bash
git add .
git commit -m "chore: initialise RepoRoller metadata repository"
git push origin main
```

---

## Step 2: Register the GitHub App

RepoRoller uses a GitHub App as its service identity for creating repositories.

1. Go to **GitHub → Organisation Settings → Developer settings → GitHub Apps → New GitHub App**.
2. Fill in the form:
   - **GitHub App name**: `RepoRoller-myorg` (must be unique on GitHub)
   - **Homepage URL**: `https://reporoller.myorg.example`
   - **Webhook**: uncheck **Active** — RepoRoller does not receive webhooks
3. Under **Repository permissions**, set:

   | Permission | Access |
   |---|---|
   | Administration | Read & write |
   | Contents | Read & write |
   | Metadata | Read-only |
   | Pull requests | Read & write |

4. Under **Organization permissions**, set:

   | Permission | Access |
   |---|---|
   | Members | Read-only |

5. Set **Where can this GitHub App be installed?** to **Only on this account**.
6. Click **Create GitHub App**.
7. Note the **App ID** shown at the top — you will need this as `GITHUB_APP_ID`.
8. Scroll to **Private keys** and click **Generate a private key**. Save the downloaded `.pem` file.
9. Click **Install App** and install into your organisation, granting access to **All repositories**.

**Convert the private key to a single-line string:**

```bash
# macOS / Linux
awk 'NF {sub(/\r/, ""); printf "%s\\n",$0;}' your-app.private-key.pem
```

Save the output as `GITHUB_APP_PRIVATE_KEY`. Treat it as a password — store it in your secrets manager.

---

## Step 3: Register the GitHub OAuth App

The frontend uses a separate OAuth App to authenticate end-users.

1. Go to **GitHub → Organisation Settings → Developer settings → OAuth Apps → New OAuth App**.
2. Fill in:
   - **Application name**: `RepoRoller`
   - **Homepage URL**: `https://reporoller.myorg.example`
   - **Authorization callback URL**: `https://reporoller.myorg.example/auth/callback`
3. Click **Register application**.
4. Copy the **Client ID** → `GITHUB_CLIENT_ID`.
5. Click **Generate a new client secret** → `GITHUB_CLIENT_SECRET`.

> **Warning:** The callback URL must exactly match the `ORIGIN` environment variable you set on the frontend container. Any mismatch causes the OAuth login to fail.

---

## Step 4: Generate secrets

Generate two independent random secrets:

```bash
# Generate SESSION_SECRET
openssl rand -hex 48

# Generate JWT_SECRET
openssl rand -hex 48
```

Record both values. They must never be the same string and must not be committed to source control.

---

## Step 5: Write the Docker Compose file

Create a directory for your deployment and write the Compose file:

```bash
mkdir ~/reporoller-deploy
cd ~/reporoller-deploy
```

```yaml
# docker-compose.yml
services:
  backend:
    image: ghcr.io/myorg/repo-roller-api:latest
    networks:
      - internal
    environment:
      GITHUB_APP_ID:          "${GITHUB_APP_ID}"
      GITHUB_APP_PRIVATE_KEY: "${GITHUB_APP_PRIVATE_KEY}"
      JWT_SECRET:             "${JWT_SECRET}"
      METADATA_REPOSITORY_NAME: ".reporoller"
      API_HOST:               "0.0.0.0"
      API_PORT:               "8080"
      RUST_LOG:               "info"
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider",
             "http://localhost:8080/api/v1/health"]
      interval: 10s
      timeout: 5s
      retries: 5

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

Create an `.env` file (do **not** commit this to source control):

```bash
cat > .env << 'EOF'
GITHUB_APP_ID=12345
GITHUB_APP_PRIVATE_KEY=-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----\n
JWT_SECRET=<your-jwt-secret>
GITHUB_CLIENT_ID=Ov23libXXXXXXXXXXXXX
GITHUB_CLIENT_SECRET=<your-oauth-client-secret>
SESSION_SECRET=<your-session-secret>
EOF
```

---

## Step 6: Start the containers

```bash
docker compose up --build -d
```

Wait about 30 seconds for both containers to become healthy, then check the status:

```bash
docker compose ps
```

You should see both services in `running (healthy)` state.

---

## Step 7: Verify the deployment

**Check the backend health endpoint:**

```bash
curl -f http://localhost:8080/api/v1/health
# 200 OK
```

**Open the frontend:**

Open `https://reporoller.myorg.example` in a browser. You should see the **Sign In** screen. Click **Sign in with GitHub** and complete the OAuth flow.

If the login succeeds and you are redirected to the **Create Repository** page, your deployment is working.

---

## Step 8: Next steps

- Add templates to your organisation so creators have choices (see [Build your first template repository](create-first-template.md))
- Configure organisation-wide defaults in `global/defaults.toml` (see [Set organisation-wide defaults](../how-to/configure/global-defaults.md))
- Set up monitoring using the Prometheus metrics the backend exposes

---

## Troubleshooting

**Sign in fails with "Access Denied"**
Verify that `GITHUB_ORG` matches the organisation slug exactly. The user must be a member of that organisation.

**OAuth callback error from GitHub**
The `Authorization callback URL` in the OAuth App settings must match `${ORIGIN}/auth/callback` exactly, including the scheme.

**Backend fails to start**
Check `GITHUB_APP_PRIVATE_KEY` is the full PEM content with literal `\n` newlines. A malformed key is the most common cause of backend startup failures. Check logs with `docker compose logs backend`.
