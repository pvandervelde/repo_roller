---
title: "Configure repository webhooks"
description: "Define GitHub repository webhooks that will be installed on every repository created from a template or configuration level."
audience: "platform-engineer"
type: "how-to"
---

# Configure repository webhooks

Repository webhooks are GitHub webhooks installed on the **created repository** itself. They are called by GitHub when events occur in that repository (pushes, pull requests, releases, etc.). They are distinct from outbound notification webhooks, which are called by RepoRoller when creation completes.

## Add webhooks to a configuration file

```toml
[[webhooks]]
name         = "ci-webhook"
url          = "https://ci.myorg.example/webhook"
content_type = "json"
secret       = "your-webhook-secret"
events       = ["push", "pull_request"]
active       = true
insecure_ssl = false
```

You may use template variables in the `url` field (template-level only):

```toml
[[webhooks]]
name         = "deployment-webhook"
url          = "https://deploy.myorg.example/webhook/{{service_name}}"
content_type = "json"
secret       = "your-webhook-secret"
events       = ["push", "release"]
active       = true
insecure_ssl = false
```

## Field reference

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Webhook name — used in logs and for identification |
| `url` | string | Yes | Endpoint URL that GitHub will POST to |
| `content_type` | string | Yes | `"json"` or `"form"` |
| `secret` | string | No | HMAC secret that GitHub uses to sign requests |
| `events` | array of string | Yes | GitHub event types (e.g. `["push", "pull_request", "release"]`) |
| `active` | bool | No | Default: `true`. Set `false` to install but not deliver events. |
| `insecure_ssl` | bool | No | Default: `false`. Set `true` only for internal endpoints with self-signed certificates. |

## Where webhooks can be defined

Repository webhooks can be defined in:

- `global/defaults.toml` — applied to every repository
- `types/{type}/config.toml` — applied to repositories of that type
- `teams/{team}/config.toml` — applied to repositories for that team
- `.reporoller/template.toml` inside a template repository — applied to repositories created from that template

Webhooks are **additive** — all webhooks from all levels are installed.

## This is not the same as outbound notification webhooks

> **Note:** Repository webhooks (`[[webhooks]]`) are installed on the created repository and fired by GitHub. Outbound notification webhooks are fired by RepoRoller itself when creation completes. See [Configure outbound notification webhooks](outbound-notifications.md) for the other type.

## Related reference

- [Configure outbound notification webhooks](outbound-notifications.md)
- [Template configuration schema](../../reference/configuration/template-config.md)
