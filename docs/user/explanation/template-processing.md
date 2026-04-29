---
title: "How templates are processed"
description: "The lifecycle of a template from selection through to the initial commit in the new repository."
audience: "platform-engineer"
type: "explanation"
---

# How templates are processed

When a user requests a repository from a template, RepoRoller carries out the following sequence. Understanding this helps you debug unexpected file content or missing files in created repositories.

## Template selection

RepoRoller searches the organisation for repositories with the `reporoller-template` GitHub topic. When the user specifies a template by name, RepoRoller looks for a repository with that name that also has the topic. If no matching repository is found, the creation request fails with a `TEMPLATE_NOT_FOUND` error.

## Cloning and file inventory

RepoRoller fetches a snapshot of the template repository's default branch. It then builds a list of all files, excluding:

- The `.reporoller/` directory entirely (it is configuration, not scaffold)
- Any files matching the `exclude_patterns` list in `[templating]`

Files that are not in the inventory are never copied to the created repository.

## Variable substitution

For each text file in the inventory, RepoRoller runs Handlebars template processing, substituting `{{variable_name}}` expressions with resolved values. Resolution order for a variable name:

1. User-supplied value (from the creation request)
2. Default value declared in `[variables]` (for optional variables with a default)
3. Built-in variable (e.g. `repo_name`, `org_name`, `creator_username`)

If a required variable has no value from the user and no default, the creation request fails before any GitHub API calls are made.

Binary files are not processed — they are copied byte-for-byte.

Files whose extension is not in `process_extensions` (when that setting is configured) are also copied without processing.

## File and directory name substitution

After content substitution, file and directory names are rewritten using the same Handlebars rules. A file named `src/{{service_name}}/main.rs` becomes `src/payment-service/main.rs` if `service_name` is `payment-service`.

The `.template` suffix is special: it is stripped from any file name. `README.md.template` becomes `README.md`. This lets the template repository contain both a developer-facing `README.md` and an output scaffold `README.md.template` without conflict.

## Repository creation and initial commit

Once processing is complete, RepoRoller:

1. Creates the GitHub repository via the GitHub App
2. Pushes all processed files as a single initial commit to the default branch
3. Applies all configuration (settings, labels, branch protection, teams, webhooks) as declared across the merged configuration hierarchy

The initial commit author is the GitHub App, not the requesting user. The requesting user's identity is captured in the audit log and in the `creator_username` built-in variable.

## What cannot be templated

- **Binary files**: copied unchanged, no variable substitution
- **The `.reporoller/` directory**: always excluded
- **Files excluded by `exclude_patterns`**: copied never — not even non-processed
- **Repository settings**: controlled by TOML configuration, not by file content
