---
title: "Use variables in file and directory names"
description: "Embed template variables in file and directory paths so created repositories have dynamically named files."
audience: "platform-engineer"
type: "how-to"
---

# Use variables in file and directory names

RepoRoller substitutes template variables in file and directory names using the same `{{variable_name}}` syntax as file content.

## Variable substitution in paths

A file named `src/{{service_name}}/main.rs` in the template becomes `src/payment-service/main.rs` in the created repository when the creator supplies `service_name = "payment-service"`.

A directory named `{{service_name}}/` becomes `payment-service/` in the output.

## The .template suffix convention

Any file with a `.template` suffix has that suffix stripped when the repository is created. This lets the template repository contain both:

- A developer-facing file (e.g. `README.md` — visible on GitHub for the template itself)
- A scaffold file that will become the output (e.g. `README.md.template` → `README.md` in created repos)

Both the `.template` suffix stripping and `{{variable}}` path substitution are applied in the same pass.

**Example tree in the template repository:**

```
.reporoller/
  template.toml
README.md                          # Template developer README — excluded from output
README.md.template                 # → README.md in created repos
src/
  {{service_name}}/
    main.rs                        # → src/payment-service/main.rs
    {{service_name}}_handler.rs    # → src/payment-service/payment-service_handler.rs
.github/
  workflows/
    ci.yml.template                # → .github/workflows/ci.yml
```

## Characters that are invalid in file names

Variable values used in file paths must not contain characters that are invalid in file names on the target operating systems. Avoid:

- `/` and `\` (path separators)
- `:`, `*`, `?`, `"`, `<`, `>`, `|` (Windows-invalid characters)
- Leading or trailing spaces or periods

Use a `pattern` constraint in your variable declaration to enforce safe values:

```toml
[variables.service_name]
description = "Name of the microservice"
required    = true
pattern     = "^[a-z][a-z0-9-]*$"
example     = "payment-service"
```

This regular expression ensures the value is lowercase alphanumeric with hyphens and starts with a letter — safe for use in file and directory names on all platforms.

## Related guides

- [Define template variables](define-template-variables.md)
- [Use variables in file content](use-variables-in-files.md)
