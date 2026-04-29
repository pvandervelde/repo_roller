---
title: "Metadata repository structure"
description: "Required and optional directory layout for the .reporoller metadata repository."
audience: "platform-engineer"
type: "reference"
---

# Metadata repository structure

The `.reporoller` metadata repository is the single source of truth for all organisation-level configuration. RepoRoller reads it on every repository creation request.

## Annotated directory tree

```
.reporoller/                       ← root of the metadata repository
│
├── global/                        ← REQUIRED — organisation-wide defaults
│   ├── defaults.toml              ← REQUIRED — global configuration (repo settings, labels, teams...)
│   └── notifications.toml         ← optional — outbound notification webhooks for all repos
│
├── types/                         ← optional — repository type configurations
│   ├── library/
│   │   └── config.toml            ← type configuration for the "library" type
│   ├── service/
│   │   └── config.toml
│   └── {type-name}/
│       └── config.toml
│
└── teams/                         ← optional — team-level configurations
    ├── backend-team/
    │   ├── config.toml            ← team configuration
    │   └── notifications.toml     ← optional — team-scoped outbound notifications
    ├── frontend-team/
    │   └── config.toml
    └── {team-name}/
        └── config.toml
```

## Required vs optional items

| Item | Required | Behaviour when absent |
|---|---|---|
| `global/` directory | Yes | Repository creation fails with a configuration error |
| `global/defaults.toml` | Yes | Repository creation fails with a configuration error |
| `types/` directory | No | No type-specific configuration is applied |
| `types/{name}/config.toml` | No | Requests specifying that type use only global defaults |
| `teams/` directory | No | No team-specific configuration is applied |
| `teams/{name}/config.toml` | No | Requests specifying that team use only global and type defaults |
| `global/notifications.toml` | No | No org-level outbound notifications |
| `teams/{name}/notifications.toml` | No | No team-level outbound notifications |

## Naming rules

| Item | Rule |
|---|---|
| Type directory names | Must match the `repository_type` slug used in creation requests and template config. Lowercase, hyphens allowed. |
| Team directory names | Must match the GitHub team slug exactly (as returned by the GitHub API). Lowercase, hyphens allowed. |
| Config file names | Must be exactly `config.toml`. Other names are ignored. |
| Notifications file names | Must be exactly `notifications.toml`. |

## Template discovery

RepoRoller discovers available templates by searching for repositories in the organisation that have the **`reporoller-template`** GitHub topic. The metadata repository itself does not need to list templates.

To register a template, add the `reporoller-template` topic to the template repository on GitHub. To unregister it, remove the topic.

## What happens on absent or malformed files

| Scenario | Behaviour |
|---|---|
| `global/defaults.toml` is missing | Hard error — repository creation fails |
| A type config file has a TOML syntax error | Hard error on any creation request for that type |
| A team config file has an unknown key | Warning logged; key ignored; creation proceeds |
| A notifications file is absent | No notifications for that level; not an error |
| A notifications file has an invalid URL | Warning logged; that endpoint skipped; others still fire |
