---
title: "Create a repository using the CLI"
description: "Walk through installing the CLI, authenticating, and creating your first repository from the command line."
audience: "repository-creator"
type: "tutorial"
---

# Create a repository using the CLI

By the end of this tutorial you will have installed the `repo-roller` CLI, authenticated it with GitHub, and used it to create a new repository from a template.

**What you need before you start:**

- A terminal (PowerShell, bash, zsh, or similar)
- Membership in the `myorg` GitHub organisation
- A GitHub personal access token or access to the organisation's CI token

---

## Step 1: Install the CLI

### Option A — cargo install

If you have the Rust toolchain installed:

```bash
cargo install repo-roller
```

### Option B — Download a release binary

1. Download the latest release for your platform from the organisation's internal release page.
2. Place the binary on your `PATH`. On macOS/Linux:

```bash
chmod +x repo-roller
sudo mv repo-roller /usr/local/bin/
```

On Windows, move `repo-roller.exe` to a directory that is on `%PATH%`.

1. Verify the installation:

```bash
repo-roller --version
```

You should see output like `repo-roller 0.5.0`.

---

## Step 2: Set your GitHub token

The CLI uses the `GITHUB_TOKEN` environment variable to authenticate with GitHub.

```bash
# macOS / Linux
export GITHUB_TOKEN="ghp_your_personal_access_token"

# PowerShell (Windows)
$env:GITHUB_TOKEN = "ghp_your_personal_access_token"
```

The token needs at minimum `repo` scope (to create repositories) and `read:org` scope (to read organisation settings). Ask your platform team if you are unsure which token to use.

---

## Step 3: Inspect the available templates

Before creating a repository, inspect the `rust-service` template to see what variables it requires:

```bash
repo-roller template info --org myorg --template rust-service
```

Output:

```
Template: rust-service

Description: Production-ready Rust microservice template
Author: Platform Team
Tags: rust, microservice, backend

Repository Type: service (policy: fixed)

Variables (2):
  ✓ service_name [required]
    Name of the microservice
    Example: user-service

  • service_port [optional]
    Port the service listens on
    Default: 8080
    Example: 3000

Configuration: 4 sections defined
```

The `✓` symbol means the variable is required. The `•` means it is optional and has a default.

---

## Step 4: Create the repository

Run the create command, providing all required variables:

```bash
repo-roller create \
  --org myorg \
  --repo my-service \
  --template rust-service \
  --description "My new Rust microservice" \
  --variable service_name="my-service" \
  --variable service_port="8080"
```

RepoRoller prints progress as it works:

```
Creating repository myorg/my-service from template rust-service...
  ✓ Repository created
  ✓ Template files applied (12 files)
  ✓ Branch protection rules applied (2 rulesets)
  ✓ Labels applied (6 labels)
  ✓ Team permissions applied

Repository created: https://github.com/myorg/my-service
```

---

## Step 5: Verify the repository

Open the URL printed in the output, or clone the repository:

```bash
git clone https://github.com/myorg/my-service.git
cd my-service
ls
```

You should see:

```
.github/
  workflows/
    ci.yml
.gitignore
Cargo.toml
README.md
src/
  main.rs
```

Open `README.md` and confirm that `my-service` appears where the template used `{{repo_name}}`. Your template variables have been substituted throughout the file tree.

---

## Step 6: Check the branch protection

On GitHub, go to **Settings → Rules → Rulesets** for the new repository. You will see the rulesets applied from the template and organisation configuration — for example, a rule preventing deletion of `main` and requiring pull requests.

**You're done.** Your repository is created, populated, and protected.

---

## Next steps

- [Create a repository from a template](../how-to/create/from-template.md) — all CLI and API options
- [Create an empty repository](../how-to/create/empty-repository.md) — for code imports
- [CLI Reference: repo-roller create](../reference/cli/create.md) — all flags and options
- [Build your first template repository](create-first-template.md) — create your own templates
