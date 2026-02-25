# RepoRoller Outbound Notification Webhooks

This guide covers configuring, consuming, and verifying outbound webhook notifications sent by
RepoRoller when repositories are created.

## Table of Contents

- [Overview](#overview)
- [Configuration File Format](#configuration-file-format)
  - [Field Reference](#field-reference)
  - [Secret Reference Format](#secret-reference-format)
  - [Event Types](#event-types)
- [Configuration Hierarchy](#configuration-hierarchy)
  - [Organization-Level](#organization-level)
  - [Team-Level](#team-level)
  - [Template-Level](#template-level)
  - [Accumulation and Deduplication](#accumulation-and-deduplication)
- [Event Payload Schema](#event-payload-schema)
  - [Full JSON Example](#full-json-example)
- [Request Signing (HMAC-SHA256)](#request-signing-hmac-sha256)
  - [HTTP Request Headers](#http-request-headers)
  - [Signature Verification](#signature-verification)
- [Secret Management](#secret-management)
  - [Environment Variables](#environment-variables)
  - [Volume Mounts (Kubernetes / Docker)](#volume-mounts-kubernetes--docker)
  - [Azure Key Vault](#azure-key-vault)
  - [AWS Secrets Manager](#aws-secrets-manager)
- [Deployment Patterns](#deployment-patterns)
  - [Azure Functions / Container Apps](#azure-functions--container-apps)
  - [AWS Lambda / ECS](#aws-lambda--ecs)
  - [Kubernetes](#kubernetes)
- [Example Webhook Receivers](#example-webhook-receivers)
- [Delivery Behaviour](#delivery-behaviour)
- [Observability](#observability)

---

## Overview

When RepoRoller successfully creates a repository it fires a `repository.created` event to all
configured webhook endpoints. Delivery is:

- **Asynchronous** – runs in a background task after repository creation returns to the caller.
- **Best-effort** – failures are logged and counted in metrics but do not affect repository
  creation outcome.
- **Signed** – every request carries an HMAC-SHA256 signature your receiver can use to verify
  authenticity.

---

## Configuration File Format

Notification endpoints are configured in `notifications.toml` files placed inside your
`.reporoller` metadata repository (see [Configuration Hierarchy](#configuration-hierarchy)).

```toml
[[outbound_webhooks]]
url             = "https://monitoring.example.com/hooks/repo-created"
secret          = "REPOROLLER_WEBHOOK_SECRET"
events          = ["repository.created"]
active          = true
timeout_seconds = 10
description     = "Central monitoring system"
```

### Field Reference

| Field            | Type            | Required | Default | Description                                           |
|------------------|-----------------|----------|---------|-------------------------------------------------------|
| `url`            | string          | ✅       | –       | Webhook endpoint URL. **Must use `https://`.**        |
| `secret`         | string          | ✅       | –       | Secret reference used for HMAC signing (see below).  |
| `events`         | array of string | ✅       | –       | Event types to deliver. Use `"*"` for all events.    |
| `active`         | bool            | ❌       | `true`  | Set to `false` to temporarily disable the endpoint.  |
| `timeout_seconds`| integer         | ❌       | `5`     | Per-request timeout. Range: 1–30 seconds.            |
| `description`    | string          | ❌       | –       | Human-readable description (not sent in requests).   |

Multiple endpoints are defined by repeating `[[outbound_webhooks]]` sections.

### Secret Reference Format

The `secret` field holds a **reference** to the actual secret value — not the secret itself.
RepoRoller resolves this reference at runtime using the configured `SecretResolver`:

| Resolver                    | `secret` value example              | How it resolves                              |
|-----------------------------|-------------------------------------|----------------------------------------------|
| `EnvironmentSecretResolver` | `"REPOROLLER_WEBHOOK_SECRET"`       | Reads `$REPOROLLER_WEBHOOK_SECRET` env var   |
| `FileSecretResolver`        | `"secrets/webhook-secret.txt"`      | Reads file at `<base_path>/secrets/webhook-secret.txt` |

See [Secret Management](#secret-management) for production patterns.

### Event Types

| Value                | Description                                   |
|----------------------|-----------------------------------------------|
| `"repository.created"` | Repository was successfully created.        |
| `"*"`                | Wildcard – matches all current and future event types. |

---

## Configuration Hierarchy

RepoRoller accumulates notification endpoints from **all configuration levels** for each
repository creation. Endpoints are additive – they are never overridden or removed by a higher
level.

### Organization-Level

**File**: `.reporoller/global/notifications.toml`

Applies to every repository created in the organization. Use for central audit logging,
monitoring dashboards, or compliance systems.

```toml
# .reporoller/global/notifications.toml

[[outbound_webhooks]]
url             = "https://audit.corp.example.com/hooks/reporoller"
secret          = "REPOROLLER_AUDIT_SECRET"
events          = ["*"]
timeout_seconds = 15
description     = "Corporate audit log"

[[outbound_webhooks]]
url             = "https://monitoring.corp.example.com/hooks/repos"
secret          = "REPOROLLER_MONITORING_SECRET"
events          = ["repository.created"]
description     = "Infrastructure monitoring"
```

### Team-Level

**File**: `.reporoller/teams/{team-name}/notifications.toml`

Applies to repositories created by or for the specified team. Use for team-specific CI
orchestration, Slack notifications, or project management integrations.

```toml
# .reporoller/teams/platform-engineering/notifications.toml

[[outbound_webhooks]]
url             = "https://ci.platform-eng.example.com/hooks/new-repo"
secret          = "PE_CI_WEBHOOK_SECRET"
events          = ["repository.created"]
description     = "Platform Engineering CI provisioner"

[[outbound_webhooks]]
url             = "https://hooks.slack.com/services/T00000/B00000/XXXX"
secret          = "PE_SLACK_WEBHOOK_SECRET"
events          = ["repository.created"]
timeout_seconds = 5
description     = "Platform Engineering Slack channel"
```

### Template-Level

**File**: `.reporoller/notifications.toml` (inside the **template** repository itself)

Applies only when this specific template is used. Use for template-specific onboarding flows,
documentation generation, or specialized tooling registration.

```toml
# .reporoller/notifications.toml (inside template repo)

[[outbound_webhooks]]
url             = "https://service-catalog.example.com/hooks/register"
secret          = "SERVICE_CATALOG_SECRET"
events          = ["repository.created"]
timeout_seconds = 10
description     = "Register new microservice in the service catalog"
```

### Accumulation and Deduplication

All endpoints from all levels are **combined**. If the same `(url, event_type)` pair appears
at multiple levels, it is **deduplicated** — only one delivery is sent per unique
`(url, event_type)`. This prevents duplicate notifications when an endpoint is configured at
both org and team level.

---

## Event Payload Schema

RepoRoller sends an HTTP `POST` request to each endpoint with a JSON body conforming to the
`RepositoryCreatedEvent` schema.

### Fields

#### Required Fields

| JSON field         | Type   | Description                                              |
|--------------------|--------|----------------------------------------------------------|
| `event_type`       | string | Always `"repository.created"`                            |
| `event_id`         | string | Unique UUID v4 for this event delivery                   |
| `timestamp`        | string | ISO 8601 UTC timestamp (e.g. `"2026-02-23T14:30:00Z"`)  |
| `organization`     | string | GitHub organization name                                 |
| `repository_name`  | string | Name of the created repository                           |
| `repository_url`   | string | Full HTTPS URL (e.g. `"https://github.com/acme/my-repo"`) |
| `repository_id`    | string | GitHub node ID (global node identifier)                  |
| `created_by`       | string | Identity of the user who requested creation              |
| `content_strategy` | string | One of `"template"`, `"empty"`, `"custom_init"`         |
| `visibility`       | string | One of `"public"`, `"private"`, `"internal"`             |

#### Optional Fields

| JSON field           | Type             | Description                                              |
|----------------------|------------------|----------------------------------------------------------|
| `repository_type`    | string \| null   | Repository type classification (e.g. `"microservice"`)  |
| `template_name`      | string \| null   | Template used; absent for empty/custom-init repos        |
| `team`               | string \| null   | Team that requested creation; absent if not team-scoped  |
| `description`        | string \| null   | Repository description; absent if not set                |
| `custom_properties`  | object \| null   | Map of custom property key→value applied to the repo     |
| `applied_settings`   | object \| null   | Repository settings applied (see sub-fields below)       |

#### `applied_settings` Sub-Fields

| JSON field        | Type          | Description                                   |
|-------------------|---------------|-----------------------------------------------|
| `has_issues`      | bool \| null  | Whether GitHub Issues is enabled               |
| `has_wiki`        | bool \| null  | Whether GitHub Wiki is enabled                 |
| `has_projects`    | bool \| null  | Whether GitHub Projects is enabled             |
| `has_discussions` | bool \| null  | Whether GitHub Discussions is enabled          |

### Full JSON Example

```json
{
  "event_type": "repository.created",
  "event_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-02-23T14:30:00.123456789Z",
  "organization": "acme-corp",
  "repository_name": "payment-service",
  "repository_url": "https://github.com/acme-corp/payment-service",
  "repository_id": "R_kgDOIa1234",
  "created_by": "jane.doe@acme-corp.com",
  "content_strategy": "template",
  "visibility": "private",
  "repository_type": "microservice",
  "template_name": "rust-service",
  "team": "payments-team",
  "description": "Payment processing microservice",
  "custom_properties": {
    "cost_center": "CC-1234",
    "service_tier": "production"
  },
  "applied_settings": {
    "has_issues": true,
    "has_wiki": false,
    "has_projects": false,
    "has_discussions": null
  }
}
```

---

## Request Signing (HMAC-SHA256)

Every webhook request is signed so your receiver can verify it originated from RepoRoller and
was not tampered with in transit.

### HTTP Request Headers

| Header                         | Value / Format                                        |
|--------------------------------|-------------------------------------------------------|
| `Content-Type`                 | `application/json`                                    |
| `User-Agent`                   | `RepoRoller/1.0`                                      |
| `X-RepoRoller-Signature-256`   | `sha256=<hex-encoded HMAC-SHA256 of the request body>` |

### Signature Algorithm

```
HMAC-SHA256(key=<resolved_secret>, message=<raw_request_body_bytes>)
```

The header value is always prefixed with `sha256=`. For example:

```
X-RepoRoller-Signature-256: sha256=3db53b367bc30614a4c20b87716c8b3f5d89...
```

### Signature Verification

To verify a request:

1. Read the raw request body bytes **before** parsing JSON.
2. Compute `HMAC-SHA256(secret_bytes, body_bytes)`.
3. Hex-encode the result and prepend `sha256=`.
4. Compare to the value in `X-RepoRoller-Signature-256` using a **constant-time** comparison.

See [Example Webhook Receivers](#example-webhook-receivers) for complete implementations in
Python, Node.js, Go, Rust, and C#.

> **Security note**: Always use constant-time comparison (e.g. `hmac.compare_digest` in Python,
> `crypto.timingSafeEqual` in Node.js, `hmac.Equal` in Go,
> `Mac::verify_slice` in Rust, `CryptographicOperations.FixedTimeEquals` in C#)
> to prevent timing attacks.

---

## Secret Management

The `secret` field in `notifications.toml` is a **reference string** — not the secret value
itself. RepoRoller resolves the reference at delivery time.

### Environment Variables

The default `EnvironmentSecretResolver` reads secrets from environment variables. The `secret`
field is used as the environment variable name:

```toml
secret = "REPOROLLER_WEBHOOK_SECRET"
```

```bash
# Set before starting RepoRoller
export REPOROLLER_WEBHOOK_SECRET="your-actual-secret-value"
```

This works for local development, simple container deployments, and Docker Compose setups.

### Volume Mounts (Kubernetes / Docker)

The `FileSecretResolver` reads secrets from files. Configure a base path and use relative
file paths as the `secret` reference:

```toml
secret = "webhook-secret"   # resolved as <base_path>/webhook-secret
```

```yaml
# Kubernetes Secret volume mount
volumes:
  - name: webhook-secrets
    secret:
      secretName: reporoller-webhook-secrets
volumeMounts:
  - name: webhook-secrets
    mountPath: /run/secrets
    readOnly: true
```

File content is read as-is (whitespace is trimmed).

### Azure Key Vault

Use Azure Managed Identity + Key Vault with the Workload Identity pattern:

```toml
# Secret name in Key Vault
secret = "reporoller-webhook-secret"
```

```yaml
# Azure Container Apps / AKS with CSI Secret Store
# azure-keyvault-secrets.yaml
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: reporoller-secrets
spec:
  provider: azure
  parameters:
    usePodIdentity: "false"
    clientID: "<managed-identity-client-id>"
    keyvaultName: "<key-vault-name>"
    objects: |
      array:
        - |
          objectName: reporoller-webhook-secret
          objectType: secret
    tenantId: "<tenant-id>"
  secretObjects:
    - secretName: reporoller-webhook-secret
      type: Opaque
      data:
        - key: reporoller-webhook-secret
          objectName: reporoller-webhook-secret
```

Use `FileSecretResolver` to read the mounted secret.

### AWS Secrets Manager

Use IAM Roles for Service Accounts (IRSA) with the Secrets Manager CSI driver or inject as
environment variables via the AWS Secrets Manager Agent:

```toml
secret = "REPOROLLER_WEBHOOK_SECRET"
```

```json
// IAM policy for the service account
{
  "Effect": "Allow",
  "Action": ["secretsmanager:GetSecretValue"],
  "Resource": "arn:aws:secretsmanager:<region>:<account>:secret:reporoller-webhook-secret-*"
}
```

```yaml
# ECS task definition environment variable from Secrets Manager
{
  "name": "REPOROLLER_WEBHOOK_SECRET",
  "valueFrom": "arn:aws:secretsmanager:<region>:<account>:secret:reporoller-webhook-secret"
}
```

---

## Deployment Patterns

### Azure Functions / Container Apps

```toml
# notifications.toml
[[outbound_webhooks]]
url             = "https://my-func-app.azurewebsites.net/api/reporoller-webhook"
secret          = "REPOROLLER_WEBHOOK_SECRET"
events          = ["repository.created"]
timeout_seconds = 10
```

```bicep
// Container App with Key Vault secret injection
resource containerApp 'Microsoft.App/containerApps@2023-05-01' = {
  properties: {
    template: {
      containers: [
        {
          env: [
            {
              name: 'REPOROLLER_WEBHOOK_SECRET'
              secretRef: 'webhook-secret'
            }
          ]
        }
      ]
    }
    configuration: {
      secrets: [
        {
          name: 'webhook-secret'
          keyVaultUrl: 'https://<vault-name>.vault.azure.net/secrets/reporoller-webhook-secret'
          identity: '<managed-identity-resource-id>'
        }
      ]
    }
  }
}
```

### AWS Lambda / ECS

```toml
# notifications.toml
[[outbound_webhooks]]
url             = "https://<api-id>.execute-api.<region>.amazonaws.com/prod/reporoller"
secret          = "REPOROLLER_WEBHOOK_SECRET"
events          = ["repository.created"]
timeout_seconds = 10
```

```yaml
# ECS task definition excerpt
containerDefinitions:
  - name: reporoller
    secrets:
      - name: REPOROLLER_WEBHOOK_SECRET
        valueFrom: arn:aws:secretsmanager:<region>:<account>:secret:reporoller-webhook-secret
```

### Kubernetes

```toml
# notifications.toml
[[outbound_webhooks]]
url             = "https://internal-service.namespace.svc.cluster.local/webhooks/reporoller"
secret          = "REPOROLLER_WEBHOOK_SECRET"
events          = ["repository.created"]
timeout_seconds = 5
```

```yaml
# Kubernetes Deployment with Secret env var
apiVersion: apps/v1
kind: Deployment
metadata:
  name: reporoller
spec:
  template:
    spec:
      containers:
        - name: reporoller
          env:
            - name: REPOROLLER_WEBHOOK_SECRET
              valueFrom:
                secretKeyRef:
                  name: reporoller-secrets
                  key: webhook-secret
```

---

## Example Webhook Receivers

Complete example implementations with HMAC signature verification:

| Language | File | Runtime |
|----------|------|---------|
| **Python** | [`examples/webhook-receiver/python/receiver.py`](../examples/webhook-receiver/python/receiver.py) | Python 3.9+ · Flask |
| **Node.js** | [`examples/webhook-receiver/node/receiver.js`](../examples/webhook-receiver/node/receiver.js) | Node.js 18+ · stdlib only |
| **Go** | [`examples/webhook-receiver/go/receiver.go`](../examples/webhook-receiver/go/receiver.go) | Go 1.21+ · stdlib only |
| **Rust** | [`examples/webhook-receiver/rust/src/main.rs`](../examples/webhook-receiver/rust/src/main.rs) | Rust stable · axum 0.7 |
| **C#** | [`examples/webhook-receiver/csharp/Program.cs`](../examples/webhook-receiver/csharp/Program.cs) | .NET 8+ · ASP.NET Core |

All examples demonstrate:

- Reading the raw request body for signature verification
- Computing HMAC-SHA256 and comparing with **constant-time equality**
- Parsing and dispatching on `event_type`
- Handling the `repository.created` payload
- Returning appropriate HTTP status codes (`204 No Content` on success)

---

## Delivery Behaviour

| Scenario                        | Behaviour                                          |
|---------------------------------|----------------------------------------------------|
| Endpoint returns 2xx            | Delivery considered successful; logged at INFO     |
| Endpoint returns 4xx / 5xx      | Delivery considered failed; logged at WARN         |
| Network error / timeout         | Delivery considered failed; logged at WARN         |
| Secret resolution fails         | Endpoint skipped; logged at WARN                   |
| `active = false`                | Endpoint skipped silently                          |
| No matching endpoints           | No HTTP requests sent; logged at INFO              |
| Serialization error             | No requests sent; logged at ERROR                  |

**Important**: All delivery failures are fire-and-forget. Repository creation is never affected
by notification failures.

---

## Observability

RepoRoller exposes Prometheus metrics for webhook delivery. See
[`event_metrics.rs`](../crates/repo_roller_core/src/event_metrics.rs) module documentation for
the full list of PromQL dashboard queries.

Key metrics:

| Metric                                  | Type      | Description                              |
|-----------------------------------------|-----------|------------------------------------------|
| `notification_delivery_attempts_total`  | Counter   | Total delivery attempts per endpoint URL |
| `notification_delivery_successes_total` | Counter   | Successful deliveries (2xx responses)    |
| `notification_delivery_failures_total`  | Counter   | Failed deliveries (non-2xx / errors)     |
| `notification_delivery_duration_ms`     | Histogram | Delivery latency distribution            |
| `notification_active_tasks`             | Gauge     | Currently running background tasks       |
