---
title: "Create the metadata repository"
description: "Create and initialise the .reporoller GitHub repository that holds all RepoRoller configuration."
audience: "operator"
type: "how-to"
---

# Create the metadata repository

## Create the repository

1. In your GitHub organisation, create a new **private** repository named `.reporoller`.
2. Do not initialise it with a README — you will push an initial commit in the next step.

## Clone and create the directory structure

```bash
git clone https://github.com/myorg/.reporoller.git
cd .reporoller

mkdir -p global types teams
```

**Required directories:**

| Directory | Purpose |
|---|---|
| `global/` | Organisation-wide defaults |
| `types/` | Repository type subdirectories |
| `teams/` | Team subdirectories |

## Create the minimal global configuration

```bash
cat > global/defaults.toml << 'EOF'
[repository]
has_issues      = true
has_projects    = false
has_wiki        = false
has_discussions = false
default_branch  = "main"

[pull_requests]
allow_merge_commit  = false
allow_squash_merge  = true
allow_rebase_merge  = false
required_approving_review_count = 1
EOF
```

## Commit and push

```bash
git add .
git commit -m "chore: initialise RepoRoller metadata repository"
git push origin main
```

## Optional: add a notifications.toml

To send webhook notifications for every repository creation, create `global/notifications.toml`:

```toml
[[outbound_webhooks]]
url             = "https://audit.myorg.example/hooks/reporoller"
secret          = "REPOROLLER_AUDIT_SECRET"
events          = ["*"]
description     = "Corporate audit log"
```

Commit and push this file as well.

## Grant the GitHub App access

After you create and install the GitHub App (see [Register the GitHub App](register-github-app.md)), ensure the App is installed with at least **read** access to the `.reporoller` repository. RepoRoller reads this repository at request time — if it is inaccessible, creation requests fail.

## Related guides

- [Set organisation-wide defaults](../configure/global-defaults.md)
- [Metadata repository structure reference](../../reference/metadata-repository.md)
