---
title: "Create your first repository in 5 minutes"
description: "Walk through creating a new GitHub repository from a template using the RepoRoller web interface."
audience: "repository-creator"
type: "tutorial"
---

# Create your first repository in 5 minutes

By the end of this tutorial you will have a new GitHub repository inside your organisation, created and fully configured from a template — in about five minutes.

**What you need before you start:**

- A browser and internet access
- Membership in the `myorg` GitHub organisation
- VPN connection to the corporate network

---

## Step 1: Sign in

1. Open your browser and navigate to the RepoRoller URL (e.g. `https://reporoller.myorg.example`).
2. You will see the **Sign In** screen with a single button: **Sign in with GitHub**.
3. Click **Sign in with GitHub**.
4. GitHub opens an OAuth authorisation page. Review the permissions and click **Authorize**.
5. You are redirected back to RepoRoller. The header now shows your GitHub avatar and username.

> **Note:** If you see an **Access Denied** page instead, you are either not a member of the `myorg` organisation or your VPN is not connected. Contact your platform team.

---

## Step 2: Open the Create Repository wizard

1. Click the **Create Repository** button in the navigation bar, or navigate to `/create`.
2. The wizard opens on **Step 1 of 3: Choose a template**.

---

## Step 3: Choose a template

The first step shows a list of available templates. Each card shows the template name, a short description, and any tags.

1. Browse the list. For this tutorial, select **rust-service** — a production-ready Rust microservice template.
2. Click the **rust-service** card to select it. It becomes highlighted.
3. Click **Next** to proceed to Step 2.

> **Tip:** If you are not sure which template to use, click a card to see a description of what files and configuration it includes before committing.

---

## Step 4: Fill in repository details

Step 2 asks for the repository name and optional settings.

1. In the **Repository name** field, type `payment-service`.
2. In the **Description** field, type `Payment processing microservice`.
3. Leave **Visibility** set to **Private** (the organisation default).
4. Leave **Repository type** set to **service** (pre-selected by the template).
5. Click **Next** to proceed to Step 3.

> **Note:** Repository names must be lowercase letters, numbers, hyphens, underscores, or periods. The field validates as you type and shows a checkmark when the name is valid and available.

---

## Step 5: Fill in template variables

Step 3 shows the variables declared by the **rust-service** template. These are substituted into file content and file names when the repository is created.

1. In the **Service name** field, type `payment-service`.
2. In the **Service port** field, type `8080` (or accept the default).
3. Review the summary on the right side of the screen:
   - Template: `rust-service`
   - Repository: `myorg/payment-service`
   - Variables: 2 provided

4. Click **Create Repository**.

---

## Step 6: Wait for creation

A progress indicator appears while RepoRoller:

1. Creates the repository in your organisation.
2. Applies the template files with your variable values substituted.
3. Applies branch-protection rules, labels, teams, and webhooks from the configuration hierarchy.

This typically takes 5–15 seconds.

---

## Step 7: See the result

The **Repository Created** screen appears with:

- A link to the new repository on GitHub: `https://github.com/myorg/payment-service`
- A summary of what was applied (labels, rulesets, team permissions)

Click **Open on GitHub** to view your new repository. You will find:

- A populated `README.md` with `payment-service` substituted where the template used `{{repo_name}}`
- A `.gitignore` tuned for Rust
- A `.github/workflows/ci.yml` CI pipeline
- Branch protection on `main` already active

**You're done.** Your repository is ready to clone and start working in.

---

## Next steps

- [Create a repository using the CLI](create-first-repository.md) — repeat this for terminal-based workflows
- [Tour of the web interface](tour-of-the-web-ui.md) — learn every screen in depth
- [Core concepts](../explanation/core-concepts.md) — understand templates, types, and the configuration hierarchy
