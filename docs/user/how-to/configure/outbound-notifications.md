---
title: "Configure outbound notification webhooks"
description: "Configure RepoRoller to send signed HMAC notifications to external systems when repositories are created."
audience: "platform-engineer"
type: "how-to"
---

# Configure outbound notification webhooks

Outbound notification webhooks are HTTP requests that **RepoRoller** sends to external systems after a repository is created. They are distinct from repository webhooks (which GitHub fires on events inside the repository).

Delivery is asynchronous (runs after creation returns), best-effort (failures are logged but do not affect creation), and signed with HMAC-SHA256.

## Configuration file placement

Create `notifications.toml` files in the metadata repository at the level(s) where you want notifications:

| Level | File path | Fires for |
|---|---|---|
| Organisation | `.reporoller/global/notifications.toml` | Every repository creation |
| Team | `.reporoller/teams/{team-name}/notifications.toml` | Repositories for that team |
| Template | `.reporoller/notifications.toml` inside the template repo | Repositories from that template |

## Configuration format

```toml
[[outbound_webhooks]]
url             = "https://monitoring.myorg.example/hooks/repo-created"
secret          = "REPOROLLER_WEBHOOK_SECRET"
events          = ["repository.created"]
active          = true
timeout_seconds = 10
description     = "Central monitoring system"

[[outbound_webhooks]]
url             = "https://audit.myorg.example/hooks/reporoller"
secret          = "REPOROLLER_AUDIT_SECRET"
events          = ["*"]
timeout_seconds = 15
description     = "Corporate audit log"
```

## Field reference

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `url` | string | Yes | — | Webhook endpoint URL. Must use `https://`. |
| `secret` | string | Yes | — | Secret reference for HMAC signing (see below). |
| `events` | array of string | Yes | — | Event types. Use `["repository.created"]` or `["*"]`. |
| `active` | bool | No | `true` | Set `false` to temporarily disable. |
| `timeout_seconds` | integer | No | `5` | Per-request timeout. Range: 1–30 seconds. |
| `description` | string | No | — | Human-readable description (not sent in requests). |

## Secret reference format

The `secret` field is a **reference** to the secret value, not the secret itself. RepoRoller resolves it at delivery time.

| Resolver | `secret` value | How it resolves |
|---|---|---|
| `EnvironmentSecretResolver` | `"REPOROLLER_WEBHOOK_SECRET"` | Reads `$REPOROLLER_WEBHOOK_SECRET` environment variable |
| `FileSecretResolver` | `"secrets/webhook-secret.txt"` | Reads file at `<base_path>/secrets/webhook-secret.txt` |

For Docker deployments, set the secret as an environment variable on the backend container:

```bash
REPOROLLER_WEBHOOK_SECRET="your-actual-secret-value"
```

## Event types

| Value | Description |
|---|---|
| `"repository.created"` | Repository was successfully created |
| `"*"` | All current and future event types |

## Accumulation and deduplication

Endpoints from all levels are combined for each creation. If the same `(url, event_type)` pair appears at multiple levels, only one delivery is sent.

## Example: organisation-level audit and team-level Slack notification

`global/notifications.toml`:

```toml
[[outbound_webhooks]]
url             = "https://audit.myorg.example/hooks/reporoller"
secret          = "REPOROLLER_AUDIT_SECRET"
events          = ["*"]
timeout_seconds = 15
description     = "Corporate audit log"
```

`teams/platform-engineering/notifications.toml`:

```toml
[[outbound_webhooks]]
url             = "https://hooks.slack.com/services/T00000/B00000/XXXX"
secret          = "PE_SLACK_WEBHOOK_SECRET"
events          = ["repository.created"]
timeout_seconds = 5
description     = "Platform Engineering Slack channel"
```

## HMAC signature verification

Every request includes `X-RepoRoller-Signature-256: sha256=<hex>`. Compute `HMAC-SHA256(secret, raw_body)` and compare using constant-time equality.

See the `examples/webhook-receiver/` directory for complete implementations in Python, Node.js, Go, Rust, and C#.

## Delivery behaviour

| Scenario | Behaviour |
|---|---|
| Endpoint returns 2xx | Success — logged at INFO |
| Endpoint returns 4xx/5xx | Failure — logged at WARN |
| Network error / timeout | Failure — logged at WARN |
| Secret resolution fails | Endpoint skipped — logged at WARN |
| `active = false` | Skipped silently |

Repository creation is never affected by delivery failures.

## Related guides

- [Configure repository webhooks](webhooks.md) — GitHub-fired webhooks on the created repo (different concept)
