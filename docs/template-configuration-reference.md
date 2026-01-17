# Template Configuration Reference

**Audience**: Template authors
**File Location**: `.reporoller/template.toml` in template repositories

---

## Overview

Every template repository must contain a `.reporoller/template.toml` file that defines the template's metadata, configuration defaults, and behavior. This file uses TOML format and supports multiple sections for different configuration aspects.

## Complete Example

```toml
# Template metadata (required)
[template]
name = "rust-microservice"
description = "Production-ready Rust microservice with observability"
author = "Platform Engineering Team"
tags = ["rust", "microservice", "backend", "production"]
default_visibility = "private"  # Optional: "public", "private", "internal"

# Repository type specification (optional)
[repository_type]
type = "service"
policy = "fixed"  # or "preferable"

# Repository feature settings (optional)
[repository]
wiki = false
issues = true
projects = true
security_advisories = true

# Pull request settings (optional)
[pull_requests]
required_approving_review_count = 2
require_code_owner_reviews = true
require_conversation_resolution = true

# Branch protection settings (optional)
[branch_protection]
enforce_admins = true
require_linear_history = true
allow_force_pushes = false

# Template variables (optional)
[variables.service_name]
description = "Name of the microservice"
example = "user-service"
required = true

[variables.service_port]
description = "HTTP port for the service"
example = "8080"
required = true
default = "8080"

# File filtering configuration (optional)
[templating]
include_patterns = [
    "**/*.rs",
    "**/*.toml",
    "**/*.md",
    "Dockerfile",
    ".dockerignore",
]
exclude_patterns = [
    "target/**",
    "**/*.log",
    "**/tmp/**",
]

# Labels (optional, additive)
[[labels]]
name = "service"
color = "0052CC"
description = "Microservice component"

[[labels]]
name = "rust"
color = "dea584"
description = "Rust codebase"

# Webhooks (optional, additive)
[[webhooks]]
url = "https://events.example.com/github"
events = ["push", "pull_request"]
active = true

# GitHub Apps (optional, additive)
[[github_apps]]
app_id = 12345
permissions = { contents = "write", pull_requests = "write" }
```

---

## Section Reference

### `[template]` Section (Required)

Metadata about the template itself.

#### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | **Yes** | Template identifier (must match repository name conventions) |
| `description` | String | **Yes** | Brief description of what this template creates |
| `author` | String | **Yes** | Team or person maintaining this template |
| `tags` | Array of Strings | **Yes** | Categorization tags for discovery |
| `default_visibility` | String | No | Default visibility: `"public"`, `"private"`, or `"internal"` |

#### Example

```toml
[template]
name = "rust-library"
description = "Reusable Rust library with CI/CD"
author = "Platform Team"
tags = ["rust", "library", "open-source"]
default_visibility = "public"
```

#### Notes

- `name` should be descriptive and follow your organization's naming conventions
- `tags` help users discover templates (use consistent taxonomy)
- `default_visibility` is subject to organization policies (see [Visibility Guide](repository-visibility.md))

---

### `[repository_type]` Section (Optional)

Specifies what repository type this template creates.

#### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | String | **Yes** | Repository type name (must exist in organization config) |
| `policy` | String | **Yes** | `"fixed"` (cannot override) or `"preferable"` (can override) |

#### Example

```toml
[repository_type]
type = "service"
policy = "fixed"
```

#### Notes

- `type` must match a repository type defined in your organization's `types/` directory
- `policy = "fixed"` prevents users from overriding the type during creation
- `policy = "preferable"` suggests this type but allows user override

---

### `[templating]` Section (Optional)

Controls which files from the template are included when creating repositories.

#### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `include_patterns` | Array of Strings | No | Glob patterns for files to include |
| `exclude_patterns` | Array of Strings | No | Glob patterns for files to exclude |

#### Example

```toml
[templating]
include_patterns = [
    "**/*.rs",
    "**/*.toml",
    "**/*.md",
]
exclude_patterns = [
    "target/**",
    "**/*.log",
]
```

#### Behavior

- If `include_patterns` is empty or omitted: All files are included
- If `include_patterns` is specified: Only matching files are included
- `exclude_patterns` always take precedence over includes
- `.reporoller/` directory is **always excluded** (hardcoded)

#### Pattern Syntax

Uses standard glob syntax:

- `*` - Any characters except `/`
- `**` - Any directories recursively
- `?` - Exactly one character
- `[abc]` - One character from set

See [Template File Filtering Guide](template-file-filtering.md) for detailed examples.

---

### `[variables.*]` Sections (Optional)

Defines variables that users must provide when creating repositories.

#### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `description` | String | **Yes** | What this variable is used for |
| `example` | String | **Yes** | Example value to guide users |
| `required` | Boolean | No | Whether this variable must be provided (default: false) |
| `default` | String | No | Default value if not provided |
| `validation_regex` | String | No | Regex pattern for validation |

#### Example

```toml
[variables.app_name]
description = "Application display name"
example = "My Awesome App"
required = true

[variables.port]
description = "HTTP port number"
example = "8080"
required = false
default = "8080"
validation_regex = "^[0-9]{4,5}$"
```

#### Variable Usage

Variables are substituted in template files using `${variable_name}` syntax:

```rust
// In template file: src/config.rs
pub const APP_NAME: &str = "${app_name}";
pub const PORT: u16 = ${port};
```

See [Template Variables Guide](template-variables.md) for full documentation.

---

### `[repository]` Section (Optional)

Repository feature settings that override organization defaults.

#### Fields

| Field | Type | Description |
|-------|------|-------------|
| `wiki` | Boolean | Enable wiki feature |
| `issues` | Boolean | Enable issue tracking |
| `projects` | Boolean | Enable project boards |
| `discussions` | Boolean | Enable discussions |
| `security_advisories` | Boolean | Enable security advisories |
| `vulnerability_reporting` | Boolean | Enable private vulnerability reporting |

#### Example

```toml
[repository]
wiki = false
issues = true
projects = true
security_advisories = true
```

---

### `[pull_requests]` Section (Optional)

Pull request and code review policies.

#### Fields

| Field | Type | Description |
|-------|------|-------------|
| `required_approving_review_count` | Integer | Number of approvals needed (0-6) |
| `require_code_owner_reviews` | Boolean | Require CODEOWNERS approval |
| `require_conversation_resolution` | Boolean | All conversations must be resolved |
| `dismiss_stale_reviews` | Boolean | Dismiss reviews on new commits |

#### Example

```toml
[pull_requests]
required_approving_review_count = 2
require_code_owner_reviews = true
require_conversation_resolution = true
```

---

### `[branch_protection]` Section (Optional)

Branch protection rules for main branch.

#### Fields

| Field | Type | Description |
|-------|------|-------------|
| `enforce_admins` | Boolean | Apply rules to administrators |
| `require_linear_history` | Boolean | Prevent merge commits |
| `allow_force_pushes` | Boolean | Allow force pushes |
| `allow_deletions` | Boolean | Allow branch deletion |
| `required_status_checks` | Array of Strings | CI checks that must pass |

#### Example

```toml
[branch_protection]
enforce_admins = true
require_linear_history = true
allow_force_pushes = false
required_status_checks = ["ci", "lint", "test"]
```

---

### `[[labels]]` Sections (Optional, Additive)

Labels to add to repositories. These are **additive** - they combine with labels from other configuration levels.

#### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | **Yes** | Label name |
| `color` | String | **Yes** | Hex color (without `#`) |
| `description` | String | No | Label description |

#### Example

```toml
[[labels]]
name = "bug"
color = "d73a4a"
description = "Something isn't working"

[[labels]]
name = "enhancement"
color = "a2eeef"
description = "New feature or request"
```

---

### `[[webhooks]]` Sections (Optional, Additive)

Webhooks to configure on repositories. Additive across configuration levels.

#### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `url` | String | **Yes** | Webhook endpoint URL |
| `events` | Array of Strings | **Yes** | GitHub events to trigger on |
| `active` | Boolean | No | Whether webhook is active (default: true) |
| `content_type` | String | No | `"json"` or `"form"` (default: "json") |

#### Example

```toml
[[webhooks]]
url = "https://ci.example.com/github-webhook"
events = ["push", "pull_request"]
active = true
content_type = "json"
```

---

### `[[github_apps]]` Sections (Optional, Additive)

GitHub Apps to install on repositories. Additive across configuration levels.

#### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `app_id` | Integer | **Yes** | GitHub App ID |
| `permissions` | Object | No | Requested permissions |

#### Example

```toml
[[github_apps]]
app_id = 12345
permissions = { contents = "write", issues = "write", pull_requests = "write" }
```

---

## Configuration Hierarchy

Template configurations have the **highest precedence** in the four-level hierarchy:

1. **Template** ← Highest priority (this file)
2. Team
3. Repository Type
4. Global Defaults ← Lowest priority

### Override Behavior

- **Simple values**: Template values override all others
- **Collections** (labels, webhooks, apps): Template values are **added** to others (additive)

### Example Hierarchy

```toml
# Global defaults define:
wiki = true

# Repository type defines:
wiki = false

# Template defines:
[repository]
wiki = true  # ← This value wins (highest priority)
```

---

## Validation

RepoRoller validates template configurations during load:

### Required Validations

- ✅ `[template]` section must exist
- ✅ `name`, `description`, `author`, `tags` must be present
- ✅ `repository_type.type` must exist in organization
- ✅ Variable names must be valid identifiers
- ✅ Glob patterns must be syntactically valid

### Warning Validations

- ⚠️ Empty `description` or `tags`
- ⚠️ No variables defined
- ⚠️ Missing examples for required variables

---

## Testing Your Configuration

### Validate Configuration

```bash
repo-roller template validate my-org my-template
```

### View Parsed Configuration

```bash
repo-roller template info my-org my-template --format json
```

### Create Test Repository

```bash
repo-roller create my-org test-repo \
  --template my-template \
  --repository-type service
```

---

## Best Practices

### ✅ Do

1. **Document everything** - Clear descriptions and examples
2. **Use semantic tags** - Consistent categorization
3. **Test thoroughly** - Create test repositories to verify
4. **Version your templates** - Use git tags for versioning
5. **Keep it simple** - Only override what's necessary
6. **Document variables** - Explain what each variable does
7. **Test filtering** - Verify files are included/excluded correctly

### ❌ Don't

1. **Don't duplicate global config** - Only override what's different
2. **Don't use platform-specific paths** - Keep patterns cross-platform
3. **Don't over-complicate** - Simple templates are easier to maintain
4. **Don't forget validation** - Always validate before using in production
5. **Don't skip documentation** - Future maintainers will thank you

---

## Troubleshooting

### "Template configuration is missing required fields"

**Solution**: Ensure `[template]` section has `name`, `description`, `author`, and `tags`.

### "Repository type 'xyz' not found"

**Solution**: The specified type doesn't exist in your organization's `types/` directory. Check available types with `repo-roller template info`.

### "Variable validation failed"

**Solution**: Provided variable values don't match `validation_regex`. Check examples and patterns.

### "No files were processed after filtering"

**Solution**: Filtering patterns are too restrictive. See [Template File Filtering Guide](template-file-filtering.md).

---

## Related Documentation

- [Template File Filtering Guide](template-file-filtering.md) - Detailed filtering patterns
- [Template Variables Guide](template-variables.md) - Variable substitution
- [Repository Visibility Guide](repository-visibility.md) - Visibility configuration
- [Creating Templates](creating-templates.md) - Template authoring guide

---

## Examples

See working examples in the [tests/templates/](../tests/templates/) directory:

- `template-test-basic` - Minimal template
- `template-test-variables` - Variable substitution
- `template-test-filtering` - File filtering
