---
title: "The metadata repository"
description: "Why organisation configuration lives in a Git repository and how to manage access to it."
audience: "platform-engineer"
type: "explanation"
---

# The metadata repository

## Why configuration lives in Git

RepoRoller stores all organisation-level configuration inside a GitHub repository — the metadata repository — rather than in a database or a static configuration file on the server.

This design choice has three significant benefits:

**Audit trail.** Every change to configuration is a Git commit made by a named person. You can trace exactly when a setting changed, who changed it, and what it was before. This is unavailable with a database UI or a config file managed outside version control.

**Change review.** Because configuration is code, it goes through the same pull request workflow that all other code changes follow. A platform engineer can propose relaxing a branch protection rule, a team lead reviews it, and the change only takes effect when it is merged. This turns policy management into a reviewable, reversible process.

**Rollback.** If a misconfiguration causes problems, reverting the change is a standard `git revert`. You do not need admin access to a database or a running service to undo a policy change.

## How RepoRoller reads the metadata repository

On every repository creation request, RepoRoller fetches the relevant configuration files from the metadata repository using the GitHub App installation token. It reads them at request time, not at startup, so configuration changes take effect immediately for the next creation — no service restart required.

The metadata repository name defaults to `.reporoller` but can be changed via the `METADATA_REPOSITORY_NAME` environment variable.

## Access control recommendations

Because the metadata repository controls security policies for all repositories in the organisation, restricting write access is important.

| Access level | Who should have it |
|---|---|
| **Write** | Platform engineers / the team responsible for organisation-wide tooling |
| **Read** | All organisation members (so everyone can understand how policies work) |
| **Admin** | Organisation owners only |

Avoid granting write access broadly. A person with write access to the metadata repository can, in principle, weaken security policies for all future repositories. Use branch protection on the metadata repository's default branch and require at least one review for pull requests.

## Naming the metadata repository

The conventional name is `.reporoller`. The leading dot makes it appear first in alphabetical listings and signals that it is an internal tooling repository. You can use any name as long as the `METADATA_REPOSITORY_NAME` environment variable on the backend matches.

## Relationship to template repositories

Template repositories are separate from the metadata repository. Each template lives in its own GitHub repository and brings its own configuration (`.reporoller/template.toml`). The metadata repository does not maintain a list of templates — RepoRoller discovers templates by searching for the `reporoller-template` GitHub topic.

This separation means:

- A team can own and evolve its own template repository without touching the metadata repository.
- Templates can be added and removed simply by adding or removing a GitHub topic.
- Policy (metadata repo) and scaffolding (template repos) have separate ownership and review flows.
