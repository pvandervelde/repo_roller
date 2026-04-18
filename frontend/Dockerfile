# =============================================================================
# Stage 1 – builder
# Install dependencies and build the SvelteKit Node.js server bundle.
# =============================================================================
FROM node:24-slim AS builder

# Install pnpm
RUN corepack enable && corepack prepare pnpm@10.33.0 --activate

WORKDIR /app

# Copy only the files needed to install dependencies so that this layer is
# cached independently of source changes.
COPY package.json pnpm-lock.yaml ./

# --ignore-scripts prevents postinstall hooks from running in CI/build contexts.
# esbuild is the only approved build script and is handled by the approvedBuilds
# list in package.json; all other lifecycle scripts are untrusted.
RUN pnpm install --frozen-lockfile --ignore-scripts

# Copy the rest of the source and build.
COPY . .

RUN pnpm run build

# Remove dev dependencies — production only from here on.
RUN pnpm prune --prod --ignore-scripts

# =============================================================================
# Stage 2 – runtime
# Minimal image that only contains the compiled server and production deps.
# =============================================================================
FROM node:24-slim AS runtime

# Run as a non-root user for security hardening.
RUN addgroup --system --gid 1001 nodejs \
    && adduser --system --uid 1001 --ingroup nodejs nodeuser

WORKDIR /app

# Copy the compiled server output and production node_modules from the builder.
COPY --from=builder /app/build ./build
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./package.json

# Drop privileges before starting the server.
USER nodeuser

# The SvelteKit Node adapter serves on port 3000 by default.
EXPOSE 3000

# ---------------------------------------------------------------------------
# Environment variables — all optional unless noted as REQUIRED
# ---------------------------------------------------------------------------
#
# Auth (REQUIRED for GitHub OAuth):
#   GITHUB_CLIENT_ID       GitHub OAuth App client ID
#   GITHUB_CLIENT_SECRET   GitHub OAuth App client secret (keep secret!)
#   GITHUB_ORG             GitHub organization slug (e.g. "my-company")
#   ORIGIN                 Public URL of this server, e.g. https://reporoller.example.com
#                          Required by the SvelteKit Node adapter for CSRF protection.
#   SESSION_SECRET         REQUIRED. Random secret (≥32 chars) used to HMAC-sign session
#                          cookies. Rotate to invalidate all live sessions.
#                          Generate: node -e "console.log(require('crypto').randomBytes(48).toString('hex'))"
#
# Backend proxy (REQUIRED for production use):
#   BACKEND_API_URL        Base URL of the Rust API server, e.g. http://api:3000
#                          All /api/v1/* requests are forwarded here in production.
#   BACKEND_API_TOKEN      Bearer token sent to the Rust API when proxying. Set this
#                          to the same value expected by the backend's auth middleware.
#
# Branding (all optional — defaults used when absent):
#   BRAND_APP_NAME          Application name shown in the header and page title
#   BRAND_LOGO_URL          URL to the light-mode logo image
#   BRAND_LOGO_URL_DARK     URL to the dark-mode logo image (requires BRAND_LOGO_URL)
#   BRAND_LOGO_ALT          Alt text for the logo (defaults to "<appName> logo")
#   BRAND_PRIMARY_COLOR     CSS colour for the brand accent (hex, e.g. #0969da)
#   BRAND_PRIMARY_COLOR_DARK  Brand accent colour override for dark mode
#
# See docs/configuration-guide.md for the full configuration reference.
# ---------------------------------------------------------------------------

ENV NODE_ENV=production

CMD ["node", "build"]
