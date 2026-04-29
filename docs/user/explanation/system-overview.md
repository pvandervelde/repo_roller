---
title: "How RepoRoller works"
description: "A user-facing overview of the three interfaces, the metadata repository, and the repository creation sequence."
audience: "all"
type: "explanation"
---

# How RepoRoller works

## What RepoRoller is

RepoRoller is an automated GitHub repository creation and configuration system. It eliminates the manual, error-prone process of copying templates, replacing placeholders, and configuring repository settings for every new project. Instead, a user fills in a short form (or runs a CLI command) and gets a fully configured, production-ready repository in seconds.

## Three ways to create a repository

RepoRoller exposes three interfaces. They share identical capabilities — all three can create repositories from templates, empty, or custom-init content strategies.

**Web UI** — a browser-based creation wizard accessible at your deployment's URL. Requires a VPN connection. Authentication via GitHub OAuth shows users their verified GitHub identity and records it in the audit log.

**CLI (`repo-roller`)** — a command-line tool for developers and automation scripts. Suitable for CI pipelines and GitHub Actions workflows. Takes a GitHub App installation token from an environment variable.

**REST API** — an HTTP/JSON API at `/api/v1`. Suitable for programmatic access from external tools. Same token-based authentication as the CLI.

## The metadata repository as source of truth

All organisation-level configuration lives in a single GitHub repository, conventionally named `.reporoller`. It contains TOML files that define baseline defaults, per-type overrides, and per-team overrides. Because configuration is in Git, every change has a commit history, can be reviewed in a pull request, and can be reverted.

Template repositories are separate — each template lives in its own GitHub repository. RepoRoller discovers templates by searching for the `reporoller-template` GitHub topic rather than maintaining a registry.

## The GitHub App as the actor

RepoRoller acts as a GitHub App when calling the GitHub API. The App behaves as a service identity: in GitHub's audit logs, repository creation and configuration appear as actions taken by the App, not by individual users. The user's identity is captured separately in RepoRoller's own audit log events.

## The creation sequence

When a repository creation request arrives, RepoRoller follows six phases:

1. **Request processing** — validate inputs (repository name format, required variables, visibility value). Fail fast before any GitHub API calls.

2. **Authentication** — verify the provided token against GitHub. Extract the organisation context.

3. **Configuration loading** — read global, type, and team config from the metadata repository. Merge into a single resolved configuration. Block on any locked-setting violations.

4. **Template processing** *(template strategy only)* — fetch the template repository snapshot. Apply file exclusions. Substitute variables into file content and file names. Strip `.template` suffixes.

5. **Repository creation** — create the GitHub repository, push the processed files as the initial commit.

6. **Configuration application** — apply repository settings, labels, branch protection rulesets, team and collaborator permissions, and webhooks. Fire outbound notification webhooks asynchronously.

At each step, a failure stops the process and returns a clear error to the user. Phases 1–3 complete without creating anything on GitHub, so failures there leave no partial state to clean up.
