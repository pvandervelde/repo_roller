# RepoRoller Configuration Guide

This guide explains how to configure RepoRoller using the hierarchical configuration system.

## Table of Contents

- [Overview](#overview)
- [Configuration Hierarchy](#configuration-hierarchy)
- [Configuration Files](#configuration-files)
  - [Global Configuration](#global-configuration)
  - [Repository Type Configuration](#repository-type-configuration)
  - [Team Configuration](#team-configuration)
  - [Template Configuration](#template-configuration)
- [Repository Settings](#repository-settings)
- [Pull Request Settings](#pull-request-settings)
- [Labels Configuration](#labels-configuration)
- [Webhooks Configuration](#webhooks-configuration)
- [Outbound Notification Webhooks](#outbound-notification-webhooks)
- [Repository Rulesets](#repository-rulesets)
- [Teams and Collaborators](#teams-and-collaborators)
- [Permission Audit Logging](#permission-audit-logging)
- [Override Controls](#override-controls)
- [Examples](#examples)
- [Web Frontend Deployment](#web-frontend-deployment)
  - [Branding Configuration](#branding-configuration)
  - [GitHub OAuth App Setup](#github-oauth-app-setup)
  - [Docker Deployment](#docker-deployment)

## Overview

RepoRoller uses a hierarchical configuration system that allows organizations to define baseline policies, teams to customize their workflows, and templates to specify their requirements. Configuration is stored in TOML files within a metadata repository (typically named `.reporoller-test` or `.reporoller`).

## Configuration Hierarchy

Configuration is resolved in the following order (highest precedence first):

1. **Template** - Defined in template repositories (`.reporoller/template.toml`)
2. **Team** - Team-specific overrides (`teams/{team-name}/config.toml`)
3. **Repository Type** - Type-specific settings (`types/{type-name}/config.toml`)
4. **Global** - Organization-wide defaults (`global/defaults.toml`)
5. **System** - Built-in fallback defaults

Higher levels override lower levels, but can be constrained by `override_allowed` controls.

## Configuration Files

### Global Configuration

Located at `global/defaults.toml` in the metadata repository.

```toml
# global/defaults.toml

[repository]
has_issues = true
has_projects = false
has_wiki = true
has_discussions = true

[pull_requests]
allow_merge_commit = false
allow_squash_merge = true
allow_rebase_merge = false
required_approving_review_count = 1
```

### Repository Type Configuration

Located at `types/{type-name}/config.toml` in the metadata repository.

```toml
# types/library/config.toml

[repository]
has_wiki = false  # Libraries typically don't need wikis
has_projects = false
security_advisories = true
vulnerability_reporting = true

[pull_requests]
required_approving_review_count = 2
require_code_owner_reviews = true
```

### Team Configuration

Located at `teams/{team-name}/config.toml` in the metadata repository.

```toml
# teams/backend-team/config.toml

[repository]
has_discussions = false  # Override global default
has_projects = true

[pull_requests]
required_approving_review_count = 2
allow_auto_merge = true
```

### Template Configuration

Located at `.reporoller/template.toml` in template repositories.

```toml
# .reporoller/template.toml

[template]
name = "rust-microservice"
description = "Production-ready Rust microservice template"
author = "Platform Team"
tags = ["rust", "microservice", "backend"]

[repository]
has_wiki = false
security_advisories = true

[pull_requests]
required_approving_review_count = 2
require_code_owner_reviews = true
```

## Repository Settings

Configure repository features and settings:

```toml
[repository]
# Feature toggles
has_issues = true
has_projects = false
has_wiki = true
has_discussions = false

# Security settings
security_advisories = true
vulnerability_reporting = true

# Repository behavior
default_branch = "main"
allow_forking = true
is_template = false
```

## Pull Request Settings

Configure pull request and merge settings:

```toml
[pull_requests]
# Merge methods
allow_merge_commit = true
allow_squash_merge = true
allow_rebase_merge = false

# Review requirements
required_approving_review_count = 2
require_code_owner_reviews = true
dismiss_stale_reviews_on_push = true
require_conversation_resolution = true

# Auto-merge
allow_auto_merge = false

# Merge commit messages
merge_commit_title = "MERGE_MESSAGE"
squash_merge_commit_title = "PR_TITLE"
```

## Labels Configuration

Define repository labels:

```toml
[[labels]]
name = "bug"
color = "d73a4a"
description = "Something isn't working"

[[labels]]
name = "enhancement"
color = "a2eeef"
description = "New feature or request"

[[labels]]
name = "documentation"
color = "0075ca"
description = "Improvements or additions to documentation"
```

## Webhooks Configuration

Configure repository webhooks:

```toml
[[webhooks]]
name = "ci-webhook"
url = "https://ci.example.com/webhook"
content_type = "json"
secret = "your-webhook-secret"
events = ["push", "pull_request"]
active = true
insecure_ssl = false
```

## Outbound Notification Webhooks

RepoRoller can send signed outbound webhook notifications to external systems when
repositories are created. These are **separate** from repository webhooks above — they are
called by RepoRoller itself, not by GitHub.

Configuration is stored in `notifications.toml` files within the metadata repository and
accumulated across the configuration hierarchy:

```toml
# global/notifications.toml — active for every repository creation
[[outbound_webhooks]]
url             = "https://monitoring.example.com/hooks/repo-created"
secret          = "REPOROLLER_WEBHOOK_SECRET"
events          = ["repository.created"]
timeout_seconds = 10
description     = "Central monitoring system"
```

| Field            | Required | Default | Description                                        |
|------------------|----------|---------|----------------------------------------------------|
| `url`            | ✅       | –       | Endpoint URL — must use `https://`                 |
| `secret`         | ✅       | –       | Environment variable name holding the signing key  |
| `events`         | ✅       | –       | Event types: `["repository.created"]` or `["*"]`  |
| `active`         | ❌       | `true`  | Set `false` to temporarily disable                 |
| `timeout_seconds`| ❌       | `5`     | Per-request timeout (1–30 seconds)                 |
| `description`    | ❌       | –       | Human-readable description                         |

For the complete reference including event payload schema, HMAC signing, secret management,
and deployment patterns, see [docs/notifications.md](notifications.md).

## Repository Rulesets

Repository rulesets provide governance rules for branches and tags.

### Basic Branch Protection

```toml
[[rulesets]]
name = "main-branch-protection"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "non_fast_forward"

[[rulesets.rules]]
type = "required_linear_history"
```

### Pull Request Requirements

```toml
[[rulesets]]
name = "pr-requirements"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main", "refs/heads/develop"]

[[rulesets.rules]]
type = "pull_request"
required_approving_review_count = 2
require_code_owner_review = true
dismiss_stale_reviews_on_push = true
allowed_merge_methods = ["squash"]
```

### Status Check Requirements

```toml
[[rulesets]]
name = "ci-checks"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "required_status_checks"
strict_required_status_checks_policy = true
required_checks = [
  { context = "ci/build" },
  { context = "ci/test" },
  { context = "security/scan" }
]
```

### Tag Protection

```toml
[[rulesets]]
name = "release-tags"
target = "tag"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/tags/v*"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "update"
```

### Advanced Ruleset Configuration

```toml
[[rulesets]]
name = "comprehensive-protection"
target = "branch"
enforcement = "active"

# Allow organization admins to bypass
[[rulesets.bypass_actors]]
actor_id = 1
actor_type = "OrganizationAdmin"
bypass_mode = "always"

[rulesets.conditions.ref_name]
include = ["refs/heads/main", "refs/heads/release/*"]
exclude = ["refs/heads/release/experimental-*"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "non_fast_forward"

[[rulesets.rules]]
type = "pull_request"
required_approving_review_count = 2
require_code_owner_review = true
require_last_push_approval = true
dismiss_stale_reviews_on_push = true
required_review_thread_resolution = true
allowed_merge_methods = ["squash", "merge"]

[[rulesets.rules]]
type = "required_status_checks"
strict_required_status_checks_policy = true
required_checks = [
  { context = "ci/build" },
  { context = "ci/test" },
  { context = "ci/integration", integration_id = 12345 },
  { context = "security/sast" },
  { context = "security/dependencies" }
]
```

### Ruleset Rule Types

Available rule types:

- **deletion**: Prevents deletion of matching references
- **non_fast_forward**: Prevents force pushes
- **required_linear_history**: Requires linear history (no merge commits)
- **update**: Prevents updates to matching references
- **pull_request**: Requires pull request before merging
- **required_status_checks**: Requires specific status checks to pass
- **required_signatures**: Requires signed commits
- **creation**: Prevents creation of matching references

### Ruleset Hierarchy

Rulesets are **additive** across the configuration hierarchy:

```
Global:      main-protection (prevents deletion)
Type:        library-protection (requires 2 approvals)
Team:        team-checks (requires specific status checks)
Template:    template-requirements (requires code owner review)

Result:      All four rulesets are applied to the repository
```

**Note:** If rulesets with the same name are defined at different levels, they will create
separate independent rulesets (not merged). A warning will be logged when duplicate names
are detected.

## Teams and Collaborators

RepoRoller can automatically assign GitHub teams and collaborators to every repository
created within an organization, using entries from the metadata repository and from the
creation request itself.

### Assignment

```toml
# global/defaults.toml

# Added to every repository at triage level.
[[default_teams]]
slug         = "reporoller-test-permissions"
access_level = "triage"

# Service account added as read-only collaborator.
[[default_collaborators]]
username     = "ci-bot"
access_level = "read"
```

Valid `access_level` values: `"none"`, `"read"`, `"triage"`, `"write"`, `"maintain"`, `"admin"`.

### Merge Algorithm

Team and collaborator entries are collected from four sources, in order of increasing
precedence:

```
1. Global defaults   global/defaults.toml        [[default_teams]] / [[default_collaborators]]
2. Repository type   types/{type}/config.toml    [[default_teams]]
3. Template          .reporoller/template.toml   [[teams]] / [[collaborators]]
4. Request           API / CLI call              teams: {slug: level} / collaborators: {user: level}
```

| Level | Locked-entry violation | Demotion attempt       | Exceeds ceiling      |
|-------|------------------------|------------------------|----------------------|
| 1–3   | Hard error (blocked)   | Hard error (blocked)   | N/A                  |
| 4     | Skipped + `WARN`       | Skipped + `WARN`       | Capped + `WARN`      |

**Algorithm (applied once per flag):**

1. Load global defaults into the access map; record `locked = true` flags.
2. For each type/template entry: hard-error on locked entries or demotions; otherwise add
   or upgrade.
3. For each request entry: skip (warn) on locked entries or demotions; cap (warn) if the
   level exceeds the org ceiling; otherwise add or upgrade.
4. Apply the resolved map to GitHub.

```
Level      Teams in map after merge
──────────────────────────────────────────────────────────
Global     security-ops=admin(locked), platform=write
Type       (none)
Template   platform upgraded → maintain
Request    backend-team=write added; security-ops change skipped (WARN)
──────────────────────────────────────────────────────────
Final      security-ops=admin, platform=maintain, backend-team=write
```

### Protection Rules

#### Locked Entries

Set `locked = true` on any default entry to make it immutable. No template or request may
change its level:

```toml
[[default_teams]]
slug         = "security-ops"
access_level = "admin"
locked       = true
```

At config-resolution time (layers 1–3) a locked-entry violation fails with
`PermissionLockedNotAllowed` and the repository is not created. At request time (layer 4)
the change is silently skipped with a `WARN` log.

#### No-Demotion Rule

Entries without `locked = true` may be upgraded but never downgraded by a higher layer:

```toml
[[default_teams]]
slug         = "platform"
access_level = "write"
# Templates may raise this to maintain/admin; they may not lower it.
```

A config-level demotion fails with `PermissionDemotionNotAllowed`. A request-level
demotion is silently skipped with a `WARN` log.

#### Access Level Ceiling

Cap what a **request** may grant. Config-established entries (global / type / template) are
not subject to the ceiling.

```toml
[permissions]
max_team_access_level         = "maintain"
max_collaborator_access_level = "write"
```

A request that exceeds the ceiling is capped at the ceiling value with a `WARN` log.

### Configuration Example

```toml
# global/defaults.toml

[[default_teams]]
slug         = "security-ops"
access_level = "admin"
locked       = true           # Never changes

[[default_teams]]
slug         = "platform"
access_level = "write"
                              # Templates may upgrade; requests may not demote

[[default_collaborators]]
username     = "ci-service-account"
access_level = "read"
locked       = true

[permissions]
max_team_access_level         = "maintain"
max_collaborator_access_level = "write"
```

```toml
# .reporoller/template.toml

# Upgrade platform for repositories that use this template.
[[teams]]
slug         = "platform"
access_level = "maintain"

# This would be a hard error at config-resolution time — commented out for illustration.
# [[teams]]
# slug         = "security-ops"
# access_level = "write"   ← PermissionLockedNotAllowed
```

### Common Scenarios

#### Security team always has admin, regardless of template or request

```toml
[[default_teams]]
slug         = "security-ops"
access_level = "admin"
locked       = true
```

Any template or request that modifies `security-ops` is blocked (hard error) or silently
ignored (request), respectively.

---

#### Platform team gets an elevated level for a specific template

```toml
# global/defaults.toml
[[default_teams]]
slug         = "platform"
access_level = "write"     # baseline for most repos

# .reporoller/template.toml
[[teams]]
slug         = "platform"
access_level = "maintain"  # elevated for repos of this type
```

Repositories created with this template grant `platform` team `maintain`. Other
repositories grant `write`. A subsequent request cannot demote the team back to `write`.

---

#### Request tries to grant admin; org ceiling blocks it

```toml
[permissions]
max_team_access_level = "maintain"
```

A creation request includes `teams: {"new-team": "admin"}`. RepoRoller caps the level at
`maintain` and logs:

```
WARN repo_roller_core: team_permission_capped slug="new-team" requested="admin" capped_at="maintain"
```

---

#### Request tries to demote a template-established level

The template grants `platform` team `maintain`. The creation request includes
`teams: {"platform": "read"}`. The demotion is skipped and `platform` retains `maintain`:

```
WARN repo_roller_core: team_permission_demotion_skipped slug="platform" current="maintain" requested="read"
```

## Permission Audit Logging

RepoRoller emits structured audit events via the [`tracing`](https://docs.rs/tracing)
framework for every significant permission decision. These events are routed through the
standard application tracing subscriber, which means they can be filtered, formatted as
JSON, and written to a dedicated audit sink.

### Enabling Audit Logs

Filter on the `repo_roller_core::permission_audit` target to capture only audit events:

```bash
# Text output — development
RUST_LOG="repo_roller_core::permission_audit=info" ./repo_roller_api

# JSON output — production / SIEM ingestion (requires a JSON tracing subscriber,
# e.g. tracing-subscriber with the json feature and EnvFilter)
RUST_LOG="repo_roller_core::permission_audit=info,warn=info" ./repo_roller_api
```

### Audit Event Reference

All events share these common structured fields:

| Field            | Type    | Description                                            |
|------------------|---------|--------------------------------------------------------|
| `organization`   | string  | GitHub organization name                               |
| `repository`     | string  | Repository name                                        |
| `requestor`      | string  | GitHub username of the caller                          |
| `emergency_access` | bool  | Whether emergency-access bypass was requested          |
| `outcome`        | string  | Event-specific value (see below)                       |

#### `outcome = "approved"` (INFO)

Emitted when the policy engine auto-approves a permission request.

Additional fields:

| Field         | Type | Description                            |
|---------------|------|----------------------------------------|
| `grant_count` | u64  | Number of permission grants produced   |

#### `outcome = "requires_approval"` (WARN)

Emitted when the policy engine determines the request needs manual approval.

Additional fields:

| Field               | Type   | Description                                        |
|---------------------|--------|----------------------------------------------------|
| `reason`            | string | Human-readable reason for the approval requirement |
| `restricted_count`  | u64    | Number of restricted permission grants             |

#### `outcome = "denied"` (WARN)

Emitted when the policy engine returns a hard error (`PermissionError`).

Additional fields:

| Field   | Type   | Description                          |
|---------|--------|--------------------------------------|
| `error` | string | Error description from `PermissionError::Display` |

#### `outcome = "applied"` (INFO)

Emitted after repository permissions are successfully applied to GitHub.

Additional fields:

| Field                   | Type | Description                                       |
|-------------------------|------|---------------------------------------------------|
| `teams_applied`         | u64  | Teams added or updated                            |
| `teams_skipped`         | u64  | Teams unchanged (already at correct level)        |
| `collaborators_applied` | u64  | Collaborators added or updated                    |
| `collaborators_removed` | u64  | Collaborators removed (access set to `none`)      |
| `collaborators_skipped` | u64  | Collaborators unchanged                           |
| `failed_teams`          | u64  | Teams that failed to apply (GitHub API errors)    |
| `failed_collaborators`  | u64  | Collaborators that failed to apply                |

### Example JSON Audit Event

With a JSON tracing subscriber (`tracing-subscriber` + `tracing-bunyan-formatter` or
similar):

```json
{
  "timestamp": "2024-03-15T10:42:07.123456Z",
  "level": "INFO",
  "target": "repo_roller_core::permission_audit",
  "message": "Repository permissions applied",
  "organization": "my-org",
  "repository": "my-new-repo",
  "requestor": "jsmith",
  "emergency_access": false,
  "outcome": "applied",
  "teams_applied": 3,
  "teams_skipped": 1,
  "collaborators_applied": 0,
  "collaborators_removed": 0,
  "collaborators_skipped": 0,
  "failed_teams": 0,
  "failed_collaborators": 0
}
```

### Protection Policy Audit Events

When a protection policy is triggered during request-phase enforcement, a `WARN` event is
emitted via the same target. Protection events are emitted by the core library itself
(not the audit logger) and include:

| Event message                           | Trigger                                               |
|-----------------------------------------|-------------------------------------------------------|
| `team_permission_locked_skip`           | Locked team entry in request — level preserved        |
| `team_permission_demotion_skipped`      | Team demotion attempt in request — level preserved    |
| `team_permission_capped`                | Team level exceeds org ceiling — capped               |
| `collaborator_permission_locked_skip`   | Locked collaborator entry in request — level preserved |
| `collaborator_permission_demotion_skipped` | Collaborator demotion attempt — level preserved    |
| `collaborator_permission_capped`        | Collaborator level exceeds ceiling — capped           |

These can be filtered alongside audit events using the same target:

```bash
RUST_LOG="repo_roller_core::permission_audit=warn" ./repo_roller_api
```

## Override Controls

Control which configuration values can be overridden at higher levels:

```toml
[repository]
# This value can be overridden by teams and templates
has_wiki = { value = true, override_allowed = true }

# This value CANNOT be overridden - it's an organization policy
security_advisories = { value = true, override_allowed = false }
```

When a setting has `override_allowed = false`, attempts to override it at team or template level will fail validation.

## Examples

### Example 1: Library Repository Configuration

```toml
# Global (global/defaults.toml)
[repository]
has_issues = true
has_wiki = true

# Repository Type (types/library/config.toml)
[repository]
has_wiki = false  # Libraries use README, not wiki
security_advisories = true

[pull_requests]
required_approving_review_count = 2

[[labels]]
name = "breaking-change"
color = "d73a4a"
description = "Breaking API changes"

[[rulesets]]
name = "library-main-protection"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "pull_request"
required_approving_review_count = 2
require_code_owner_review = true
```

**Result**: Library repositories have issues enabled, wiki disabled, security advisories enabled, require 2 approving reviews, have a "breaking-change" label, and protect the main branch with deletion prevention and pull request requirements.

### Example 2: Microservice Template Configuration

```toml
# Template (.reporoller/template.toml)
[template]
name = "rust-microservice"
description = "Production-ready Rust microservice"
author = "Platform Team"
tags = ["rust", "microservice", "backend"]

[repository]
has_wiki = false
has_discussions = false

[variables]
service_name = { description = "Name of the microservice", example = "user-service" }
service_port = { description = "Port the service runs on", default = "8080" }

[[webhooks]]
name = "deployment-webhook"
url = "https://deploy.example.com/webhook/{{service_name}}"
content_type = "json"
events = ["push", "release"]
active = true

[[rulesets]]
name = "microservice-ci-checks"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "required_status_checks"
strict_required_status_checks_policy = true
required_checks = [
  { context = "rust/build" },
  { context = "rust/test" },
  { context = "rust/clippy" },
  { context = "docker/build" },
  { context = "security/cargo-audit" }
]
```

**Result**: Microservice repositories from this template have a deployment webhook configured with service-specific URL, and require all Rust CI checks to pass before merging to main.

### Example 3: Team-Specific Configuration

```toml
# Team (teams/frontend-team/config.toml)
[repository]
has_discussions = true  # Frontend team likes discussions for design decisions

[pull_requests]
required_approving_review_count = 1  # Smaller team, 1 approval is sufficient
allow_auto_merge = true

[[labels]]
name = "ui"
color = "0e8a16"
description = "UI/UX changes"

[[labels]]
name = "accessibility"
color = "f9d0c4"
description = "Accessibility improvements"

[[rulesets]]
name = "frontend-ci-checks"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main", "refs/heads/develop"]

[[rulesets.rules]]
type = "required_status_checks"
required_checks = [
  { context = "ci/build" },
  { context = "ci/test" },
  { context = "ci/lint" },
  { context = "ci/visual-regression" }
]
```

**Result**: Frontend team repositories have discussions enabled, require only 1 approval (overriding global default), allow auto-merge, have UI/accessibility labels, and require visual regression tests.

---

## Web Frontend Deployment

The RepoRoller web frontend is a SvelteKit application compiled to a standalone Node.js server
using `@sveltejs/adapter-node`. It can be deployed as a Docker container alongside the Rust API
server.

### Branding Configuration

The frontend supports custom branding through two mechanisms (highest priority first):

1. **Environment variables** — set at container startup time
2. **`brand.toml`** — a TOML config file in the server's working directory

> **Security**: `brand.toml` must **not** be placed inside `frontend/static/`. It is a
> server-side file that is never served to browsers. Place it next to the built server
> (e.g. `/app/brand.toml` inside the container, or mount it as a volume).

#### Branding Environment Variables

| Variable                 | Description                                                      | Default        |
|--------------------------|------------------------------------------------------------------|----------------|
| `BRAND_APP_NAME`         | Application name in the header and page title                    | `RepoRoller`   |
| `BRAND_LOGO_URL`         | URL for the light-mode logo image                                | *(none)*       |
| `BRAND_LOGO_URL_DARK`    | URL for the dark-mode logo (requires `BRAND_LOGO_URL`)           | *(none)*       |
| `BRAND_LOGO_ALT`         | Alt text for the logo image                                      | `<name> logo`  |
| `BRAND_PRIMARY_COLOR`    | CSS accent colour (hex, rgb, etc.)                               | `#0969da`      |
| `BRAND_PRIMARY_COLOR_DARK` | Accent colour override for dark mode                           | *(none)*       |

#### `brand.toml` Example

Copy `frontend/brand.toml.example` to `brand.toml` and customise:

```toml
app_name = "Acme RepoRoller"
logo_url = "https://cdn.acme.example/logo.svg"
logo_url_dark = "https://cdn.acme.example/logo-dark.svg"
logo_alt = "Acme logo"
primary_color = "#e63946"
```

---

### GitHub OAuth App Setup

The frontend uses GitHub's OAuth flow for user authentication. You must create a GitHub OAuth
App in your organization before deploying.

#### Steps

1. Go to **GitHub → Settings → Developer settings → OAuth Apps → New OAuth App** (or your
   organization's equivalent path).
2. Fill in the form:
   - **Application name**: `RepoRoller` (or your branded name)
   - **Homepage URL**: your public URL, e.g. `https://reporoller.acme.example`
   - **Authorization callback URL**: `https://reporoller.acme.example/auth/callback`
3. Click **Register application**.
4. Copy the **Client ID** and generate a **Client secret**.
5. Set the following environment variables when running the frontend container:

| Variable               | Description                                           |
|------------------------|-------------------------------------------------------|
| `GITHUB_CLIENT_ID`     | OAuth App client ID (from step 4)                     |
| `GITHUB_CLIENT_SECRET` | OAuth App client secret — treat as a password         |
| `GITHUB_ORG`           | GitHub organization slug (e.g. `acme`)                |
| `ORIGIN`               | Public URL of the frontend, e.g. `https://reporoller.acme.example` — required by the SvelteKit Node adapter for CSRF protection |

> **Security**: `GITHUB_CLIENT_SECRET` is sensitive. Pass it via a secrets manager or
> `--env-file` rather than embedding it in container images or Docker Compose files
> committed to source control.

---

### Docker Deployment

The frontend ships with a multi-stage `Dockerfile` at `frontend/Dockerfile`.

#### Build and run

```bash
# Build the image from the frontend directory
docker build -t repo-roller-frontend frontend/

# Run with the required environment variables
docker run -d \
  --name repo-roller-frontend \
  -p 3000:3000 \
  -e GITHUB_CLIENT_ID=Iv1.abc123 \
  -e GITHUB_CLIENT_SECRET=<secret> \
  -e GITHUB_ORG=my-org \
  -e ORIGIN=https://reporoller.example.com \
  -e BRAND_APP_NAME="My RepoRoller" \
  repo-roller-frontend
```

To use `brand.toml` instead of individual environment variables, mount it as a volume at
`/app/brand.toml`:

```bash
docker run -d \
  --name repo-roller-frontend \
  -p 3000:3000 \
  -e GITHUB_CLIENT_ID=Iv1.abc123 \
  -e GITHUB_CLIENT_SECRET=<secret> \
  -e GITHUB_ORG=my-org \
  -e ORIGIN=https://reporoller.example.com \
  -v /etc/reporoller/brand.toml:/app/brand.toml:ro \
  repo-roller-frontend
```

#### Docker Compose example

```yaml
services:
  backend:
    image: repo-roller-api:latest
    ports:
      - "8080:8080"
    environment:
      GITHUB_APP_ID: ${GITHUB_APP_ID}
      GITHUB_APP_PRIVATE_KEY: ${GITHUB_APP_PRIVATE_KEY}
      GITHUB_ORG: ${GITHUB_ORG}
      # API_TOKEN is a shared secret validated by the backend middleware.
      # It must be a strong random string (not a GitHub token).
      API_TOKEN: ${BACKEND_API_TOKEN}
    restart: unless-stopped

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      GITHUB_CLIENT_ID: ${GITHUB_CLIENT_ID}
      GITHUB_CLIENT_SECRET: ${GITHUB_CLIENT_SECRET}
      GITHUB_ORG: ${GITHUB_ORG}
      ORIGIN: ${ORIGIN}
      BRAND_APP_NAME: ${BRAND_APP_NAME:-RepoRoller}
      # BACKEND_API_URL points to the backend service defined above.
      BACKEND_API_URL: http://backend:8080
      # BACKEND_API_TOKEN must match the API_TOKEN set on the backend service.
      BACKEND_API_TOKEN: ${BACKEND_API_TOKEN}
    volumes:
      - /etc/reporoller/brand.toml:/app/brand.toml:ro
    restart: unless-stopped
    depends_on:
      - backend
```

> **Note:** `BACKEND_API_TOKEN` is a shared secret between the frontend and
> backend — it is **not** a GitHub token. Generate a strong random value
> (e.g. `openssl rand -hex 32`) and keep it in your `.env` file, which must
> not be committed to source control.

Use a `.env` file (not committed to source control) to supply all secret values locally.

## Best Practices

1. **Start with sensible global defaults** that work for most repositories
2. **Use repository types** to group similar repositories with shared requirements
3. **Keep team configurations minimal** - only override what's necessary
4. **Document template requirements** clearly in template descriptions
5. **Use descriptive names** for labels, webhooks, and rulesets
6. **Test rulesets** in "evaluate" mode before switching to "active"
7. **Avoid overly restrictive rules** that might hinder productivity
8. **Use `override_allowed = false`** for critical security policies
9. **Leverage variables** in webhooks and other configuration for flexibility
10. **Keep configuration DRY** by using the hierarchy effectively

## Troubleshooting

### Configuration Not Applied

- Check the configuration hierarchy - higher levels override lower levels
- Verify `override_allowed` settings - some values cannot be overridden
- Check for TOML syntax errors in configuration files
- Ensure the metadata repository is accessible and correctly configured

### Rulesets Not Working

- Verify `enforcement` is set to "active" (not "disabled" or "evaluate")
- Check branch/tag patterns in `conditions.ref_name.include`
- Ensure rule types are spelled correctly
- Verify required status check contexts match your CI system

### Override Violations

- Check if the global configuration has `override_allowed = false` for that setting
- Verify repository type policies if using type-based configuration
- Review template configuration for conflicting requirements

## Additional Resources

- [GitHub Rulesets Documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets)
- [Outbound Notification Webhooks Guide](notifications.md)
- [Organization Repository Settings Design](docs/spec/design/organization-repository-settings.md)
- [Configuration Interfaces](docs/spec/interfaces/configuration-interfaces.md)
