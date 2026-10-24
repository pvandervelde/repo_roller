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
      LOG_FORMAT:               "json"
      OTEL_TRACES_ENABLED:      "false"
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
      OTEL_TRACES_ENABLED:  "false"
      PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT: ""

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
| `LOG_FORMAT` | No | `json` | Log output format: `json` (default, structured, with automatic redaction of sensitive fields — recommended for containers) or `pretty` (human-readable, for local development only) |
| `OTEL_TRACES_ENABLED` | No | `false` | Enables OpenTelemetry span export over OTLP. Fail-safe-off — only `true`/`1` enables it. See [Distributed Tracing](#distributed-tracing-jaeger) below. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | No | `http://localhost:4317` | OTLP/gRPC collector endpoint. Use `https://` for TLS. |
| `OTEL_SERVICE_NAME` | No | `repo-roller-api` | Service name attached to exported spans' resource attributes. |

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
| `OTEL_TRACES_ENABLED` | No | `false` | Enables OpenTelemetry span export from the SvelteKit **server** process. Fail-safe-off, same matching rule as the backend variable of the same name. See [Distributed tracing](#distributed-tracing-jaeger) below. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | No | `http://localhost:4317` | OTLP/gRPC endpoint the SvelteKit **server** process exports spans to. Ignored unless `OTEL_TRACES_ENABLED` is `true`/`1`. |
| `OTEL_SERVICE_NAME` | No | `repo-roller-frontend` | Service name attached to spans exported by the SvelteKit server process. |
| `PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT` | No | *(unset)* | OTLP/**HTTP** endpoint the **browser** exports spans to directly. Fail-safe-off — leave unset to keep browser tracing off entirely. Must be publicly reachable from end-user browsers (not the Docker-internal address used by `OTEL_EXPORTER_OTLP_ENDPOINT`). |

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

## Logging

The backend emits JSON-formatted structured logs to stdout by default, one JSON object per
line — suitable for collection by a container log driver and ingestion into a log
aggregation system. Every request handled by the backend gets a unique `request_id`, which is
attached to every log line produced while that request is being processed and is also
returned to the client as the `X-Request-Id` response header, so you can correlate a specific
client-reported failure with its server-side log lines.

All log output is automatically scanned and redacted before it is written: known-sensitive
field names (tokens, passwords, secrets, etc.) are always redacted, and — independently — any
string value that *looks like* a secret (a GitHub token, a `Bearer` header, a JWT, or a
generic high-entropy string) is redacted too, even if it appears inside a free-text log
message rather than a named field. Redacted values are replaced with the literal
`[REDACTED]`. This is defense-in-depth on top of (not a replacement for) never passing raw
secrets to a logging call.

```bash
# View backend logs (JSON lines)
docker compose logs -f backend
```

Set `LOG_FORMAT=pretty` (see the environment variable table above) if you need
human-readable output while debugging locally — this is not recommended for production
containers, since most log aggregation systems expect the structured JSON format.

### Frontend logging

The frontend's SvelteKit server process also emits structured JSON lines (one JSON object per
line: `level`, `message`, `time`, plus any caller-supplied metadata) to stdout, rather than
unstructured `console.*` output — collect it the same way as the backend's logs:

```bash
docker compose logs -f frontend
```

`Error` values passed as metadata are serialized as `{name, message, stack}` rather than
silently collapsing to `{}`, and circular references or function values embedded in metadata
are neutralized (replaced with `'[Circular]'` / a `[Function: ...]` placeholder) instead of
crashing the log call. There is no `LOG_FORMAT` toggle for the frontend — output is always
structured JSON. This logging is separate from the frontend's OpenTelemetry tracing (below);
logs and traces are two independent signals, correlated only by timestamp and, once a trace is
open in Jaeger, by the request path being investigated.

## Metrics (Prometheus)

The backend exposes a Prometheus-format scrape endpoint at `GET /metrics` on the same port as
the API (`8080` by default) — **not** under the `/api/v1` prefix. It reports request rates and
latencies per endpoint, repository-creation outcomes, and GitHub API call/error counts.

Like `/health`, `/metrics` is **unauthenticated** by design (Prometheus does not send an
`Authorization` header by default), so it must never be reachable outside the private Docker
network — the same "do not expose port 8080 externally" rule from [Networking](#networking)
above applies.

**Point Prometheus at the backend container** (adjust the target for your network/service
discovery mechanism):

```yaml
scrape_configs:
  - job_name: repo_roller_api
    static_configs:
      - targets: ["backend:8080"] # reachable on the internal Docker network only
    metrics_path: /metrics
```

**Verify locally** (from inside the `internal` network, e.g. via `docker compose exec`):

```bash
curl http://backend:8080/metrics
```

You should see Prometheus text-format output, including families such as
`repository_creation_requests_total`, `github_api_calls_total`, and `http_requests_total`. See
the [`repo_roller_api` crate README](../../../../crates/repo_roller_api/README.md#metrics) for
the full list of exposed metrics and their labels.

## Distributed tracing (Jaeger)

The backend can export OpenTelemetry traces over OTLP for the full repository-creation flow
(authentication, configuration resolution, template processing, GitHub repository creation,
and every underlying GitHub API call). This is **disabled by default** (fail-safe-off) — no
background exporter and no network I/O happen unless you explicitly opt in with
`OTEL_TRACES_ENABLED=true`.

### Running locally with the bundled Jaeger backend

`examples/local-stack/docker-compose.yml` in the repository already wires up a Jaeger
all-in-one container (OTLP-enabled) alongside the backend and frontend, so you get a working
trace UI with no extra setup:

```bash
cd examples/local-stack
cp .env.example .env   # fill in credentials, then:
OTEL_TRACES_ENABLED=true docker compose up --build
```

Open the Jaeger UI at `http://localhost:16686`, select the `repo-roller-api` service, and
search for traces after making a request (e.g. `POST /api/v1/repositories`). Each trace shows
the `repository_creation` span with its `authentication` / `configuration_resolution` /
`template_processing` / `github_repository_creation` child spans, plus GitHub API calls
nested under whichever phase made them.

### End-to-end tracing: browser → SvelteKit server → Rust API

The frontend participates in the same trace rather than producing a disconnected one. When
both `OTEL_TRACES_ENABLED=true` (frontend server) and a non-empty
`PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT` (browser) are set, a single user action — e.g. submitting
the `/create` wizard — produces **one continuous trace** spanning:

1. The browser page-load/form-submission span (exported over OTLP/HTTP to
   `PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT`).
2. A `sveltekit.request` span representing the SvelteKit server's own hop, which is a *child*
   of the browser's span (the server extracts the browser's `traceparent` header rather than
   minting a new root — exported over OTLP/gRPC to `OTEL_EXPORTER_OTLP_ENDPOINT`).
3. The Rust backend's `http_request` span and everything nested under it (`repository_creation`
   and its phase/GitHub-API children), which is in turn a child of the SvelteKit server's span
   — the backend extracts the `traceparent` header the SvelteKit server sets on its proxied
   call.

Every hop propagates a standard W3C `traceparent` header on its outbound request
(`lib/api/client.ts`'s `apiFetch`, `lib/proxy.server.ts`'s `proxyToBackend`, and the Rust
backend's `tracing_middleware`/`extract_trace_context` all participate), so searching Jaeger
for the trace ID shown in the browser's devtools network panel — or simply opening the newest
trace for the `repo-roller-frontend` or `repo-roller-api` service after making a request —
surfaces the whole chain as one trace, not three separate ones.

If either `OTEL_TRACES_ENABLED` (server) or `PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT` (browser) is
left unset, that hop's spans are simply not exported (fail-safe-off) — the remaining hops still
trace correctly on their own, they just won't have that hop's spans attached to the same trace.

To exercise this locally with `examples/local-stack`, set all four variables referenced above
before starting the stack:

```bash
cd examples/local-stack
cp .env.example .env   # fill in credentials, then:
OTEL_TRACES_ENABLED=true \
PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318/v1/traces \
docker compose up --build
```

(`examples/local-stack/docker-compose.yml` already wires these through to both the backend and
frontend services, with the Jaeger container's OTLP/HTTP receiver published on `4318` for the
browser's benefit, alongside its OTLP/gRPC receiver on `4317` used by both server processes.)

### Adding a Jaeger service to your own docker-compose.yml

If you are not using `examples/local-stack`, add a Jaeger (or other OTLP-compatible) service
and point the backend at it:

```yaml
services:
  backend:
    # ...existing configuration from above...
    environment:
      # ...existing variables...
      OTEL_TRACES_ENABLED:         "true"
      OTEL_EXPORTER_OTLP_ENDPOINT: "http://jaeger:4317"
      OTEL_SERVICE_NAME:           "repo-roller-api"
    depends_on:
      jaeger:
        condition: service_started

  jaeger:
    image: jaegertracing/all-in-one:1.62
    ports:
      - "16686:16686" # Jaeger UI
      - "4317:4317"   # OTLP gRPC receiver
    environment:
      COLLECTOR_OTLP_ENABLED: "true"
```

### Pointing at a different collector (production)

Set `OTEL_EXPORTER_OTLP_ENDPOINT` to your OTel Collector's or managed tracing backend's OTLP
endpoint instead of a local Jaeger container. Use an `https://` URL for any endpoint reachable
outside a private network — OTLP export supports TLS via the exporter's `tls-aws-lc` feature,
consistent with the backend's existing `aws-lc-rs` crypto provider. No further configuration
is required beyond the URL scheme; if the exporter fails to construct (e.g. an unreachable or
malformed endpoint), the backend logs an error and continues running with tracing disabled
rather than failing to start.

### Sanitization guarantee

Business-context span attributes (`organization`, `template`, `requestor`, and equivalents
recorded elsewhere in the GitHub API client) are sanitized before being attached to a span —
values that look like tokens, `Bearer` headers, JWTs, or high-entropy secrets are replaced
with `[REDACTED]` in their entirety, and all other values are length-bounded. This is a
separate mechanism from, but provides the same guarantee as, the log-line redaction described
in [Logging](#logging) above: neither log output nor exported trace data ever carries a raw
secret or an unbounded attacker-controlled string.

See the [`repo_roller_api` crate README: Distributed Tracing](../../../../crates/repo_roller_api/README.md#distributed-tracing)
for the full span hierarchy, environment variable reference, and sanitization details.

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
