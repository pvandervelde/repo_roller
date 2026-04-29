---
title: "Build your first template repository"
description: "Walk through scaffolding a template repository with make-template, declaring variables, and registering the template for use."
audience: "platform-engineer"
type: "tutorial"
---

# Build your first template repository

By the end of this tutorial you will have a fully scaffolded template repository on GitHub that other developers can use with `repo-roller create`. You will run `make-template`, declare a template variable, use it in a file, validate the template, and register it with your organisation.

**What you need before you start:**

- The `repo-roller` CLI installed ([Install instructions](create-first-repository.md#step-1-install-the-cli))
- A GitHub repository to convert into a template — or create a new one:

```bash
mkdir rust-service-template
cd rust-service-template
git init
git remote add origin https://github.com/myorg/rust-service-template.git
```

- Write access to that repository

---

## Step 1: Run make-template

From inside the repository directory, run:

```bash
repo-roller make-template . \
  --name "rust-service" \
  --description "Production-ready Rust microservice with gRPC and observability" \
  --author "Platform Team"
```

The command prints a preview of every file it will create:

```
Initializing template in: .

  The following files will be affected:

  CREATE     .reporoller/template.toml
  CREATE     README.md  [template developer docs — excluded from output repos]
  CREATE     README.md.template  [scaffold for repos created from this template → renamed to README.md]
  CREATE     .gitignore  [template developer gitignore — excluded from output repos]
  CREATE     .gitignore.template  [starter gitignore for new repos → renamed to .gitignore]
  CREATE     .github/workflows/test-template.yml  [validates template structure in CI — excluded from output repos]
  CREATE     .github/workflows/ci.yml.template  [CI scaffold for new repos → renamed to ci.yml]

Proceed? [y/N]:
```

Type `y` and press Enter.

---

## Step 2: Review the generated files

Open `.reporoller/template.toml`. You will find the `[template]` section already filled in with the name, description, and author you provided, and every other section commented out with inline documentation. The top of the file looks like this:

```toml
[template]
name        = "rust-service"
description = "Production-ready Rust microservice with gRPC and observability"
author      = "Platform Team"
tags        = []
```

Open `README.md.template`. This file will become `README.md` in every repository created from your template. It already contains `{{repo_name}}` and `{{template_name}}` placeholders.

---

## Step 3: Declare a variable

Repository creators need to supply a service name and an optional port. Open `.reporoller/template.toml` and add a `[variables]` section:

```toml
[template]
name        = "rust-service"
description = "Production-ready Rust microservice with gRPC and observability"
author      = "Platform Team"
tags        = ["rust", "microservice", "backend"]

[variables.service_name]
description = "Name of the microservice (lowercase, hyphens allowed)"
required    = true
example     = "payment-service"
pattern     = "^[a-z][a-z0-9-]*$"
min_length  = 3
max_length  = 63

[variables.service_port]
description = "Port the service listens on"
required    = false
default     = "8080"
example     = "3000"
```

Save the file.

---

## Step 4: Use the variable in a file

Open `README.md.template` and update it to use your new variable:

```markdown
# {{repo_name}}

{{description}}

## Service details

- **Service name**: `{{service_name}}`
- **Default port**: `{{service_port}}`
- **Template**: `{{template_name}}`

## Getting started

```bash
cargo run
```

```

The `{{service_name}}` and `{{service_port}}` placeholders will be replaced with the values the creator provides when they run `repo-roller create`.

---

## Step 5: Commit and push

```bash
git add .reporoller/ README.md README.md.template .gitignore .gitignore.template \
        .github/
git commit -m "chore: initialise RepoRoller template scaffold"
git push origin main
```

---

## Step 6: Register the template

RepoRoller discovers templates by GitHub topic. Add the `reporoller-template` topic to the repository:

1. On GitHub, go to the repository's main page.
2. Click the gear icon next to **About**.
3. In the **Topics** field, type `reporoller-template` and press Enter.
4. Click **Save changes**.

---

## Step 7: Validate the template

Run the validate command to confirm there are no configuration errors:

```bash
repo-roller template validate --org myorg --template rust-service-template
```

Expected output:

```
Validating template: rust-service-template

✓ Template repository accessible
✓ Configuration file found (.reporoller/template.toml)
✓ Template metadata complete
✓ Variables valid (2 defined)
✓ Repository type references valid

Template is VALID
```

If you see any errors, check the template manifest against the [Template manifest reference](../reference/template-authoring/template-manifest.md).

---

## Step 8: Create a test repository

Verify the full workflow end-to-end:

```bash
repo-roller create \
  --org myorg \
  --repo test-from-rust-service \
  --template rust-service-template \
  --variable service_name="test-from-rust-service" \
  --variable service_port="9090"
```

Open `https://github.com/myorg/test-from-rust-service` and confirm that `README.md` contains `test-from-rust-service` and `9090` where the placeholders were.

Delete the test repository when you are done.

---

## Next steps

- [Define template variables](../how-to/author-templates/define-template-variables.md) — full variable field reference
- [Use variables in file content](../how-to/author-templates/use-variables-in-files.md) — Handlebars syntax
- [Configure template labels](../how-to/author-templates/configure-template-labels.md) — add labels to created repos
- [Template manifest reference](../reference/template-authoring/template-manifest.md) — every field in template.toml
