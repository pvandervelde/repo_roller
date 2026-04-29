---
title: "RepoRoller User Documentation"
description: "Everything you need to create repositories, author templates, configure policies, and operate RepoRoller."
---

# RepoRoller User Documentation

RepoRoller is an internal GitHub App that lets your organisation create new GitHub repositories from curated templates with consistent security policies, branch-protection rules, team permissions, and labels — all enforced automatically. It has three interfaces: a **web UI** for point-and-click creation, a **CLI** (`repo-roller`) for scripted and terminal workflows, and a **REST API** for programmatic integration.

> **Note:** RepoRoller is internal tooling. You must be a member of the GitHub organisation and connected to the corporate VPN to access the web interface and API.

---

## What do you want to do?

### New user

| Goal | Start here |
|---|---|
| Understand what RepoRoller does | [How RepoRoller works](explanation/system-overview.md) |
| Learn the key concepts | [Core concepts](explanation/core-concepts.md) |
| Create your first repository (web UI) | [Create your first repository in 5 minutes](tutorials/quickstart.md) |
| Create your first repository (CLI) | [Create a repository using the CLI](tutorials/create-first-repository.md) |
| Tour the web interface | [Tour of the web interface](tutorials/tour-of-the-web-ui.md) |

### Create a repository

| Goal | Start here |
|---|---|
| Create from a template | [Create a repository from a template](how-to/create/from-template.md) |
| Create an empty repository | [Create an empty repository](how-to/create/empty-repository.md) |
| Create with README and .gitignore | [Create a repository with README and .gitignore](how-to/create/with-readme-and-gitignore.md) |
| Use the REST API | [Create a repository using the REST API](how-to/create/using-the-rest-api.md) |
| Automate in CI | [Automate repository creation with the CLI](how-to/integrate/automate-with-cli.md) |

### Author templates

| Goal | Start here |
|---|---|
| Build my first template | [Build your first template repository](tutorials/create-first-template.md) |
| Scaffold a template repository | [Create a template repository](how-to/author-templates/create-template-repository.md) |
| Add variables | [Define template variables](how-to/author-templates/define-template-variables.md) |
| Use variables in files | [Use variables in file content](how-to/author-templates/use-variables-in-files.md) |
| Use variables in filenames | [Use variables in file and directory names](how-to/author-templates/use-variables-in-filenames.md) |
| Register a template | [Register a template with the metadata repository](how-to/configure/register-template.md) |
| Validate a template | [Validate the metadata repository configuration](how-to/configure/validate-configuration.md) |

### Configure policies

| Goal | Start here |
|---|---|
| Set organisation-wide defaults | [Set organisation-wide defaults](how-to/configure/global-defaults.md) |
| Configure a repository type | [Configure a repository type](how-to/configure/repository-types.md) |
| Configure team settings | [Configure team-level settings](how-to/configure/team-configuration.md) |
| Configure branch protection | [Configure branch protection rules](how-to/configure/branch-protection-rules.md) |
| Configure labels | [Configure repository labels](how-to/configure/labels.md) |
| Configure webhooks | [Configure repository webhooks](how-to/configure/webhooks.md) |
| Configure outbound notifications | [Configure outbound notification webhooks](how-to/configure/outbound-notifications.md) |
| Configure team permissions | [Configure team and collaborator permissions](how-to/configure/team-permissions.md) |
| Understand the configuration hierarchy | [How configuration is resolved](explanation/configuration-hierarchy.md) |

### Deploy and operate

| Goal | Start here |
|---|---|
| Deploy for the first time | [Set up RepoRoller for your organization](tutorials/set-up-reporoller.md) |
| Create the metadata repository | [Create the metadata repository](how-to/deploy/create-metadata-repository.md) |
| Register the GitHub App | [Register the GitHub App](how-to/deploy/register-github-app.md) |
| Register the OAuth App | [Register the GitHub OAuth App](how-to/deploy/register-oauth-app.md) |
| Deploy with Docker Compose | [Deploy with Docker Compose](how-to/deploy/deploy-with-docker.md) |
| Customise branding | [Configure branding](how-to/deploy/configure-branding.md) |
| Environment variable reference | [Environment variables](reference/environment-variables.md) |

---

## Reference documentation

- [CLI Reference](reference/cli/index.md)
- [REST API Reference](reference/api/index.md)
- [Configuration Reference](reference/configuration/index.md)
- [Metadata repository structure](reference/metadata-repository.md)
- [Template authoring reference](reference/template-authoring/index.md)
- [Environment variables](reference/environment-variables.md)
