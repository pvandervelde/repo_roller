---
title: "Add configuration to a template"
description: "Configure repository settings, pull request policies, labels, webhooks, rulesets, and permissions via template.toml."
audience: "platform-engineer"
type: "how-to"
---

# Add configuration to a template

The `.reporoller/template.toml` file inside a template repository controls both template metadata and the configuration applied to every repository created from this template. Template-level settings take the highest precedence in the configuration hierarchy (overriding global, type, and team settings, subject to `override_allowed` constraints).

## Complete microservice template example

```toml
[template]
name        = "rust-microservice"
description = "Production-ready Rust microservice with gRPC and observability"
author      = "Platform Team"
tags        = ["rust", "microservice", "backend", "grpc"]

[repository]
has_wiki        = false
has_discussions = false
security_advisories      = true
vulnerability_reporting  = true

[repository.repository_type]
type_name = "service"
policy    = "fixed"      # Creators cannot override this type

[pull_requests]
required_approving_review_count = 2
require_code_owner_reviews      = true
dismiss_stale_reviews_on_push   = true

[variables.service_name]
description = "Canonical name of the microservice"
required    = true
pattern     = "^[a-z][a-z0-9-]*$"
example     = "payment-service"

[variables.service_port]
description = "HTTP port the service listens on"
required    = false
default     = "8080"
example     = "3000"

[[labels]]
name        = "performance"
color       = "f9d0c4"
description = "Performance changes"

[[teams]]
slug         = "platform"
access_level = "maintain"

[[collaborators]]
username     = "deploy-bot"
access_level = "write"

[[webhooks]]
name         = "deployment-webhook"
url          = "https://deploy.myorg.example/webhook/{{service_name}}"
content_type = "json"
events       = ["push", "release"]
active       = true
insecure_ssl = false

[[rulesets]]
name        = "microservice-ci-checks"
target      = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "required_status_checks"
strict_required_status_checks_policy = true
required_checks = [
  { context = "ci/build" },
  { context = "ci/test" },
  { context = "docker/build" },
  { context = "security/cargo-audit" }
]

[templating]
exclude_patterns = [
  "README.md",
  ".gitignore",
  ".github/workflows/test-template.yml"
]
process_extensions = [
  "rs", "toml", "md", "yml", "yaml", "json", "txt"
]
```

## Key sections

| Section | Effect |
|---|---|
| `[template]` | Metadata — name, description, author, tags |
| `[repository]` | Repository feature settings; `repository_type` controls the type policy |
| `[pull_requests]` | PR merge and review settings |
| `[variables.*]` | User-supplied variables at creation time |
| `[[labels]]` | Labels applied to the created repository |
| `[[teams]]` | Team permissions applied to the created repository |
| `[[collaborators]]` | Individual collaborator permissions |
| `[[webhooks]]` | GitHub repository webhooks installed on the created repository |
| `[[rulesets]]` | Branch/tag protection rulesets |
| `[templating]` | Controls which files are processed and which are excluded |

## Related reference

- [Template configuration schema](../../reference/configuration/template-config.md)
- [Branch protection rules](branch-protection-rules.md)
- [Team and collaborator permissions](team-permissions.md)
