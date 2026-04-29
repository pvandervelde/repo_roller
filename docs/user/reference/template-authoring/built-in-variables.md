---
title: "Built-in template variables"
description: "Variables automatically available in every template without declaration."
audience: "platform-engineer"
type: "reference"
---

# Built-in template variables

RepoRoller injects two sets of variables automatically at repository creation time. They are available in file content and file/directory names without any declaration in `[variables]`.

## Request variables

These variables describe the current creation request:

| Variable | Type | Description | Example value |
|---|---|---|---|
| `repo_name` | string | Repository name as supplied in the creation request | `payment-service` |
| `org_name` | string | GitHub organisation slug | `myorg` |
| `template_name` | string | Name of the template being used (from `[template].name`) | `rust-service` |
| `template_repo` | string | Full `owner/repo` path of the template repository | `myorg/rust-service-template` |
| `user_login` | string | GitHub login of the person who submitted the creation request | `jane.doe` |
| `user_name` | string | GitHub display name of the requester | `Jane Doe` |
| `default_branch` | string | Default branch created with the repository | `main` |
| `timestamp` | string | RFC 3339 UTC timestamp at the moment processing began | `2026-04-27T14:30:00+00:00` |
| `timestamp_unix` | string | Unix epoch seconds at the moment processing began | `1745763000` |

## Configuration variables

These variables reflect the merged organisation configuration so templates can adapt to org-wide policies. They all use the `config_` prefix.

### Repository features

| Variable | Type | Description |
|---|---|---|
| `config_issues_enabled` | `"true"` / `"false"` | Whether the Issues feature is enabled |
| `config_projects_enabled` | `"true"` / `"false"` | Whether the Projects feature is enabled |
| `config_discussions_enabled` | `"true"` / `"false"` | Whether the Discussions feature is enabled |
| `config_wiki_enabled` | `"true"` / `"false"` | Whether the Wiki feature is enabled |
| `config_pages_enabled` | `"true"` / `"false"` | Whether the Pages feature is enabled |
| `config_security_advisories_enabled` | `"true"` / `"false"` | Whether security advisories are enabled |
| `config_vulnerability_reporting_enabled` | `"true"` / `"false"` | Whether private vulnerability reporting is enabled |
| `config_auto_close_issues_enabled` | `"true"` / `"false"` | Whether issues are auto-closed on PR merge |

### Pull request settings

| Variable | Type | Description |
|---|---|---|
| `config_required_approving_review_count` | string (number) | Number of required approving reviews (omitted when not set) |
| `config_allow_merge_commit` | `"true"` / `"false"` | Whether merge commits are allowed |
| `config_allow_squash_merge` | `"true"` / `"false"` | Whether squash merges are allowed |
| `config_allow_rebase_merge` | `"true"` / `"false"` | Whether rebase merges are allowed |
| `config_allow_auto_merge` | `"true"` / `"false"` | Whether auto-merge is allowed |
| `config_delete_branch_on_merge` | `"true"` / `"false"` | Whether the source branch is deleted after merge |

## Usage example

```markdown
<!-- README.md -->
# {{repo_name}}

Created from the `{{template_name}}` template.

Organisation: `{{org_name}}`
Created by: @{{user_login}}
```

```yaml
# .github/workflows/ci.yml
{{#if config_issues_enabled}}
# Issue tracking is enabled for this repository
{{/if}}
```

## Notes

- Built-in variables are always available and cannot be overridden by user input.
- User-declared variables (defined in `[variables]`) must use different names; avoid names that clash with built-in variables.
- `timestamp` is in RFC 3339 format with the UTC offset `+00:00`. `timestamp_unix` is plain decimal seconds.
- Configuration boolean variables are always the strings `"true"` or `"false"`, never TOML booleans.
